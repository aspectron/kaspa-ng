use std::time::Duration;

use crate::imports::*;
use crate::runtime::Service;
pub use futures::{future::FutureExt, select, Future};
use kaspa_metrics::{Metric, Metrics, MetricsSnapshot};
#[allow(unused_imports)]
use kaspa_wallet_core::rpc::{NotificationMode, Rpc, RpcCtl, WrpcEncoding};
use kaspa_wallet_core::{ConnectOptions, ConnectStrategy};

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        #[cfg(not(target_arch = "wasm32"))]
        use kaspa_rpc_service::service::RpcCoreService;

        const LOG_BUFFER_LINES: usize = 4096;
        const LOG_BUFFER_MARGIN: usize = 128;
    }
}

#[allow(clippy::identity_op)]
pub const MAX_METRICS_SAMPLES: usize = 60 * 60 * 24 * 1; // 1 day

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        use std::path::PathBuf;

        pub mod config;
        pub use config::Config;
        pub mod daemon;
        pub mod inproc;
        pub mod logs;
        use logs::Log;
        pub use kaspad_lib::args::Args;

        #[async_trait]
        pub trait Kaspad {
            async fn start(&self, config : Config) -> Result<()>;
            async fn stop(&self) -> Result<()>;
            // async fn halt(&self) -> Result<()>;
        }

        #[derive(Debug, Clone)]
        pub enum KaspadServiceEvents {
            StartInternalInProc { config: Config, network : Network },
            StartInternalAsDaemon { config: Config, network : Network },
            StartExternalAsDaemon { path: PathBuf, config: Config, network : Network },
            StartRemoteConnection { rpc_config : RpcConfig, network : Network },
            Stdout { line : String },
            Stop,
            Exit,
        }

        // pub static UPDATE_LOGS_UX : Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
        pub fn update_logs_flag() -> &'static Arc<AtomicBool> {
            static FLAG: OnceLock<Arc<AtomicBool>> = OnceLock::new();
            FLAG.get_or_init(||Arc::new(AtomicBool::new(false)))
        }

        pub fn update_metrics_flag() -> &'static Arc<AtomicBool> {
            static FLAG: OnceLock<Arc<AtomicBool>> = OnceLock::new();
            FLAG.get_or_init(||Arc::new(AtomicBool::new(false)))
        }

    } else {

        #[derive(Debug)]
        pub enum KaspadServiceEvents {
            StartRemoteConnection { rpc_config : RpcConfig, network : Network },
            Stop,
            Exit,
        }

    }
}

pub struct KaspaService {
    pub application_events: ApplicationEventsChannel,
    pub service_events: Channel<KaspadServiceEvents>,
    pub task_ctl: Channel<()>,
    pub network: Mutex<Network>,
    pub wallet: Arc<runtime::Wallet>,
    pub metrics: Arc<Metrics>,
    pub metrics_data: Mutex<HashMap<Metric, Vec<PlotPoint>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub kaspad: Mutex<Option<Arc<dyn Kaspad + Send + Sync + 'static>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub logs: Mutex<Vec<Log>>,
    // pub logs: Mutex<VecDeque<Log>>,
}

impl KaspaService {
    pub fn new(application_events: ApplicationEventsChannel, settings: &Settings) -> Self {
        // --
        // create wallet instance
        let storage = runtime::Wallet::local_store().unwrap_or_else(|e| {
            panic!("Failed to open local store: {}", e);
        });

        let wallet =
            runtime::Wallet::try_with_rpc(None, storage, Some(settings.node.network.into()))
                .unwrap_or_else(|e| {
                    panic!("Failed to create wallet instance: {}", e);
                });

        // create service event channel
        let service_events = Channel::unbounded();

        // enqueue startup event to the service channel to
        // start kaspad or initiate connection to remote kaspad
        if settings.initialized {
            match KaspadServiceEvents::try_from(&settings.node) {
                Ok(event) => {
                    service_events.sender.try_send(event).unwrap_or_else(|err| {
                        println!("KaspadService error: {}", err);
                    });
                }
                Err(err) => {
                    println!("KaspadServiceEvents::try_from() error: {}", err);
                }
            }
        }

        let metrics = Arc::new(Metrics::default());
        let metrics_data = Metric::list()
            .into_iter()
            .map(|metric| (metric, Vec::new()))
            .collect::<HashMap<Metric, Vec<_>>>();

        Self {
            application_events,
            service_events,
            task_ctl: Channel::oneshot(),
            network: Mutex::new(settings.node.network),
            wallet: Arc::new(wallet),
            metrics,
            metrics_data: Mutex::new(metrics_data),
            #[cfg(not(target_arch = "wasm32"))]
            kaspad: Mutex::new(None),
            #[cfg(not(target_arch = "wasm32"))]
            logs: Mutex::new(Vec::new()),
            // logs: Mutex::new(VecDeque::new()),
        }
    }

    pub fn create_rpc_client(config: &RpcConfig, network: Network) -> Result<Rpc> {
        match config {
            RpcConfig::Wrpc { url, encoding } => {
                log_warning!("create_rpc_client - RPC URL: {:?}", url);

                let url = KaspaRpcClient::parse_url(
                    // url.as_ref().unwrap_or("127.0.0.1"), //Some(url.clone()),
                    url.clone().or(Some("127.0.0.1".to_string())),
                    *encoding,
                    NetworkId::from(network).into(),
                )?
                .expect("Unable to parse RPC URL");

                let wrpc_client = Arc::new(KaspaRpcClient::new_with_args(
                    *encoding,
                    NotificationMode::MultiListeners,
                    url.as_str(),
                )?);
                let rpc_ctl = wrpc_client.ctl().clone();
                let rpc_api: Arc<DynRpcApi> = wrpc_client;
                Ok(Rpc::new(rpc_api, rpc_ctl))
            }
            RpcConfig::Grpc { url: _ } => {
                unimplemented!("GPRC is not currently supported")
            }
        }
    }

    pub async fn connect_rpc_client(&self) -> Result<()> {
        if let Ok(wrpc_client) = self
            .wallet()
            .rpc_api()
            .clone()
            .downcast_arc::<KaspaRpcClient>()
        {
            let options = ConnectOptions {
                block_async_connect: false,
                strategy: ConnectStrategy::Retry,
                url: None,
                connect_timeout: None,
                retry_interval: Some(Duration::from_millis(3000)),
            };
            wrpc_client.connect(options).await?;
        } else {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if self
                    .wallet()
                    .rpc_api()
                    .clone()
                    .downcast_arc::<RpcCoreService>()
                    .is_ok()
                {
                    self.wallet().rpc_ctl().signal_open().await?;
                } else {
                    unimplemented!("connect_rpc_client(): RPC client is not supported")
                }
            }
        }

        Ok(())
    }

    pub fn wallet(&self) -> Arc<runtime::Wallet> {
        self.wallet.clone()
    }

    #[cfg(not(target_arch = "wasm32"))]
    // pub fn logs(&self) -> MutexGuard<'_, VecDeque<Log>> {
    pub fn logs(&self) -> MutexGuard<'_, Vec<Log>> {
        self.logs.lock().unwrap()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn update_logs(&self, line: String) {
        {
            let mut logs = self.logs.lock().unwrap();
            if logs.len() > LOG_BUFFER_LINES {
                logs.drain(0..LOG_BUFFER_MARGIN);
            }
            // logs.push_back(line.as_str().into());
            logs.push(line.as_str().into());
        }

        if update_logs_flag().load(Ordering::SeqCst) {
            self.application_events
                .sender
                .send(crate::events::Events::UpdateLogs)
                .await
                .unwrap();
        }
    }

    pub async fn stop_all_services(&self) -> Result<()> {
        if !self.wallet().has_rpc() {
            return Ok(());
        }

        if let Ok(wrpc_client) = self
            .wallet()
            .rpc_api()
            .clone()
            .downcast_arc::<KaspaRpcClient>()
        {
            wrpc_client.disconnect().await?;
        } else {
            self.wallet().rpc_ctl().signal_close().await?;
        }

        self.wallet().stop().await.expect("Unable to stop wallet");
        self.wallet().bind_rpc(None).await?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let kaspad = self.kaspad.lock().unwrap().take();
            if let Some(kaspad) = kaspad {
                println!("*** STOPPING KASPAD ***");
                if let Err(err) = kaspad.stop().await {
                    println!("error stopping kaspad: {}", err);
                }
                println!("*** KASPAD STOPPED ***");
            }
        }

        self.metrics.unregister_sink();
        self.metrics.stop_task().await?;
        self.metrics.set_rpc(None);

        Ok(())
    }

    pub async fn start_all_services(self: &Arc<Self>, rpc: Rpc, network: Network) -> Result<()> {
        let rpc_api = rpc.rpc_api().clone();

        self.wallet()
            .set_network_id(network.into())
            .expect("Can not change network id while the wallet is connected");

        // let wrpc_client = rpc.rpc_api().clone().downcast_arc::<KaspaRpcClient>().ok();

        self.wallet().bind_rpc(Some(rpc)).await.unwrap();
        self.wallet()
            .start()
            .await
            .expect("Unable to start wallet service");

        let this = self.clone();
        self.metrics
            .register_sink(Arc::new(Box::new(move |snapshot: MetricsSnapshot| {
                if let Err(err) = this.ingest_metrics_snapshot(Box::new(snapshot)) {
                    println!("Error ingesting metrics snapshot: {}", err);
                }
                None
            })));
        self.reset_metrics_data()?;
        self.metrics.start_task().await?;
        self.metrics.set_rpc(Some(rpc_api));

        // if rpc client is KaspaRpcClient, auto-connect to the node
        // if let Some(wrpc_client) = wrpc_client {
        //     let options = ConnectOptions {
        //         block_async_connect: false,
        //         strategy: ConnectStrategy::Retry,
        //         url : None,
        //         connect_timeout: None,
        //         retry_interval: Some(Duration::from_millis(3000)),
        //     };
        //     wrpc_client.connect(options).await?;
        // }

        Ok(())
    }

    pub fn update_services(&self, node_settings: &NodeSettings) {
        match KaspadServiceEvents::try_from(node_settings) {
            Ok(event) => {
                self.service_events
                    .sender
                    .try_send(event)
                    .unwrap_or_else(|err| {
                        println!("KaspadService error: {}", err);
                    });
            }
            Err(err) => {
                println!("KaspadServiceEvents::try_from() error: {}", err);
            }
        }
    }

    pub fn metrics_data(&self) -> MutexGuard<'_, HashMap<Metric, Vec<PlotPoint>>> {
        self.metrics_data.lock().unwrap()
    }

    pub fn metrics(&self) -> &Arc<Metrics> {
        &self.metrics
    }

    // pub fn connected_peer_info(&self) -> Option<Arc<Vec<RpcPeerInfo>>> {
    //     self.metrics.connected_peer_info()
    // }

    pub fn reset_metrics_data(&self) -> Result<()> {
        let now = unixtime_as_millis_f64();
        let mut template = Vec::with_capacity(MAX_METRICS_SAMPLES);
        let mut plot_point = PlotPoint {
            x: now - MAX_METRICS_SAMPLES as f64 * 1000.0,
            y: 0.0,
        };
        while template.len() < MAX_METRICS_SAMPLES {
            template.push(plot_point);
            plot_point.x += 1000.0;
        }

        let mut metrics_data = self.metrics_data.lock().unwrap();
        for metric in Metric::list().into_iter() {
            metrics_data.insert(metric, template.clone());
        }
        Ok(())
    }

    pub fn ingest_metrics_snapshot(&self, snapshot: Box<MetricsSnapshot>) -> Result<()> {
        let timestamp = snapshot.unixtime;
        let mut metrics_data = self.metrics_data.lock().unwrap();
        for metric in Metric::list().into_iter() {
            let dest = metrics_data.get_mut(&metric).unwrap();
            if dest.len() > MAX_METRICS_SAMPLES {
                dest.drain(0..dest.len() - MAX_METRICS_SAMPLES);
            }
            // else if dest.len() < MAX_METRICS_SAMPLES {
            //     let mut last_point = dest.last().cloned().unwrap_or_default();
            //     while dest.len() < MAX_METRICS_SAMPLES {
            //         last_point.x += 1000.0;
            //         dest.push(last_point.clone());
            //     }
            // }
            dest.push(PlotPoint {
                x: timestamp,
                y: snapshot.get(&metric),
            });
        }

        // if update_metrics_flag().load(Ordering::SeqCst) {
        self.application_events
            .sender
            .try_send(crate::events::Events::Metrics { snapshot })
            .unwrap();
        // }

        Ok(())
    }
}

#[async_trait]
impl Service for KaspaService {
    async fn spawn(self: Arc<Self>) -> Result<()> {
        let this = self.clone();
        let wallet_events = this.wallet.multiplexer().channel();
        let _application_events_sender = self.application_events.sender.clone();

        loop {
            // println!("loop...");
            select! {

                msg = wallet_events.recv().fuse() => {
                    if let Ok(event) = msg {
                        use kaspa_wallet_core::events::Events as CoreWallet;

                        match *event {
                            CoreWallet::DAAScoreChange{ .. } => {
                            }
                            _ => {
                                println!("wallet event: {:?}", event);
                            }
                        }
                        // if !matches(event, CoreWallet::DAAScoreChange{ .. }) {
                        // }
                        this.application_events.sender.send(crate::events::Events::Wallet{event}).await.unwrap();
                    } else {
                        break;
                    }
                }

                msg = this.as_ref().service_events.receiver.recv().fuse() => {

                    if let Ok(event) = msg {

                        match event {

                            #[cfg(not(target_arch = "wasm32"))]
                            KaspadServiceEvents::Stdout { line } => {

                                if !this.wallet().utxo_processor().is_synced() {
                                    this.wallet().utxo_processor().sync_proc().handle_stdout(&line).await?;
                                }

                                this.update_logs(line).await;
                            }

                            #[cfg(not(target_arch = "wasm32"))]
                            KaspadServiceEvents::StartInternalInProc { config, network } => {

                                this.stop_all_services().await?;

                                println!("*** STARTING NEW IN-PROC KASPAD ***");
                                let kaspad = Arc::new(inproc::InProc::default());
                                this.kaspad.lock().unwrap().replace(kaspad.clone());

                                kaspad.start(config).await.unwrap();

                                let rpc_api = kaspad.rpc_core_services().expect("Unable to obtain inproc rpc api");
                                let rpc_ctl = RpcCtl::new();
                                let rpc = Rpc::new(rpc_api, rpc_ctl.clone());

                                this.start_all_services(rpc, network).await?;
                                this.connect_rpc_client().await?;

                            },
                            #[cfg(not(target_arch = "wasm32"))]
                            KaspadServiceEvents::StartInternalAsDaemon { config, network } => {
                                self.stop_all_services().await?;

                                println!("*** STARTING NEW INTERNAL KASPAD DAEMON ***");
                                let kaspad = Arc::new(daemon::Daemon::new(None, &this.service_events));
                                this.kaspad.lock().unwrap().replace(kaspad.clone());
                                kaspad.start(config).await.unwrap();

                                let rpc_config = RpcConfig::Wrpc {
                                    url : None,
                                    encoding : WrpcEncoding::Borsh,
                                };

                                let rpc = Self::create_rpc_client(&rpc_config, network).expect("Kaspad Service - unable to create wRPC client");
                                this.start_all_services(rpc, network).await?;
                                this.connect_rpc_client().await?;
                            },
                            #[cfg(not(target_arch = "wasm32"))]
                            KaspadServiceEvents::StartExternalAsDaemon { path, config, network } => {
                                self.stop_all_services().await?;

                                println!("*** STARTING NEW EXTERNAL KASPAD DAEMON ***");
                                let kaspad = Arc::new(daemon::Daemon::new(Some(path), &this.service_events));
                                this.kaspad.lock().unwrap().replace(kaspad.clone());
                                kaspad.start(config).await.unwrap();

                                let rpc_config = RpcConfig::Wrpc {
                                    url : None,
                                    encoding : WrpcEncoding::Borsh,
                                };

                                let rpc = Self::create_rpc_client(&rpc_config, network).expect("Kaspad Service - unable to create wRPC client");
                                this.start_all_services(rpc, network).await?;
                                this.connect_rpc_client().await?;
                            },
                            KaspadServiceEvents::StartRemoteConnection { rpc_config, network } => {

                                self.stop_all_services().await?;

                                println!("*** STARTING NEW KASPAD RPC CONNECTION ***");
                                let rpc = Self::create_rpc_client(&rpc_config, network).expect("Kaspad Service - unable to create wRPC client");
                                this.start_all_services(rpc, network).await?;
                                this.connect_rpc_client().await?;

                            },

                            KaspadServiceEvents::Stop => {
                                self.stop_all_services().await?;
                            }

                            KaspadServiceEvents::Exit => {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        println!("shutting down node manager...");
        this.stop_all_services().await?;
        this.task_ctl.send(()).await.unwrap();

        Ok(())
    }

    fn terminate(self: Arc<Self>) {
        self.service_events
            .sender
            .try_send(KaspadServiceEvents::Exit)
            .unwrap();
    }

    async fn join(self: Arc<Self>) -> Result<()> {
        self.task_ctl.recv().await.unwrap();
        Ok(())
    }
}

impl TryFrom<&NodeSettings> for KaspadServiceEvents {
    type Error = Error;
    fn try_from(node_settings: &NodeSettings) -> std::result::Result<Self, Self::Error> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {

                match &node_settings.node_kind {
                    KaspadNodeKind::Disable => {
                        Ok(KaspadServiceEvents::Stop)
                    }
                    KaspadNodeKind::IntegratedInProc => {
                        // let config = ;
                        Ok(KaspadServiceEvents::StartInternalInProc { config : Config::from(node_settings.clone()), network : node_settings.network })
                    }
                    KaspadNodeKind::IntegratedAsDaemon => {
                        Ok(KaspadServiceEvents::StartInternalAsDaemon { config : Config::from(node_settings.clone()), network : node_settings.network })
                    }
                    KaspadNodeKind::ExternalAsDaemon => {
                        let path = node_settings.kaspad_daemon_binary.clone().ok_or(Error::MissingExternalKaspadBinary)?;
                        Ok(KaspadServiceEvents::StartExternalAsDaemon { path : PathBuf::from(path), config : Config::from(node_settings.clone()), network : node_settings.network })
                    }
                    KaspadNodeKind::Remote => {
                        Ok(KaspadServiceEvents::StartRemoteConnection { rpc_config : node_settings.into(), network : node_settings.network })
                    }
                }

            } else {

                match &node_settings.node_kind {
                    KaspadNodeKind::Disable => {
                        Ok(KaspadServiceEvents::Stop)
                    }
                    KaspadNodeKind::Remote => {
                        Ok(KaspadServiceEvents::StartRemoteConnection { rpc_config : node_settings.into(), network : node_settings.network })
                    }
                }
            }
        }
    }
}
