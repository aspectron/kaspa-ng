use crate::imports::*;
use crate::runtime::Service;
pub use futures::{future::FutureExt, select, Future};
#[allow(unused_imports)]
use kaspa_wallet_core::rpc::{
    ConnectOptions, ConnectStrategy, NotificationMode, Rpc, RpcCtl, WrpcEncoding,
};

const ENABLE_PREEMPTIVE_DISCONNECT: bool = true;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        #[cfg(not(target_arch = "wasm32"))]
        use kaspa_rpc_service::service::RpcCoreService;

        const LOG_BUFFER_LINES: usize = 4096;
        const LOG_BUFFER_MARGIN: usize = 128;
    }
}

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
            async fn start(self : Arc<Self>, config : Config) -> Result<()>;
            async fn stop(self : Arc<Self>) -> Result<()>;
        }

        #[derive(Debug, Clone)]
        pub enum KaspadServiceEvents {
            StartInternalInProc { config: Config, network : Network },
            StartInternalAsDaemon { config: Config, network : Network },
            StartExternalAsDaemon { path: PathBuf, config: Config, network : Network },
            StartRemoteConnection { rpc_config : RpcConfig, network : Network },
            Stdout { line : String },
            Disable { network : Network },
            Exit,
        }

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
            Disable { network : Network },
            Exit,
        }

    }
}

pub struct KaspaService {
    pub application_events: ApplicationEventsChannel,
    pub service_events: Channel<KaspadServiceEvents>,
    pub task_ctl: Channel<()>,
    pub network: Mutex<Network>,
    pub wallet: Arc<CoreWallet>,
    pub services_start_instant: Mutex<Option<Instant>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub kaspad: Mutex<Option<Arc<dyn Kaspad + Send + Sync + 'static>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub logs: Mutex<Vec<Log>>,
}

impl KaspaService {
    pub fn new(application_events: ApplicationEventsChannel, settings: &Settings) -> Self {
        // --
        // create wallet instance
        let storage = CoreWallet::local_store().unwrap_or_else(|e| {
            panic!("Failed to open local store: {}", e);
        });

        let wallet = CoreWallet::try_with_rpc(None, storage, Some(settings.node.network.into()))
            .unwrap_or_else(|e| {
                panic!("Failed to create wallet instance: {}", e);
            });

        // create service event channel
        let service_events = Channel::unbounded();

        // enqueue startup event to the service channel to
        // start kaspad or initiate connection to remote kaspad
        if settings.initialized {
            match KaspadServiceEvents::from_node_settings(&settings.node, None) {
                Ok(event) => {
                    service_events.sender.try_send(event).unwrap_or_else(|err| {
                        log_error!("KaspadService error: {}", err);
                    });
                }
                Err(err) => {
                    log_error!("KaspadServiceEvents::try_from() error: {}", err);
                }
            }
        } else {
            // log_warning!("Node settings are not initialized");
        }

        Self {
            application_events,
            service_events,
            task_ctl: Channel::oneshot(),
            network: Mutex::new(settings.node.network),
            wallet: Arc::new(wallet),
            services_start_instant: Mutex::new(None),
            #[cfg(not(target_arch = "wasm32"))]
            kaspad: Mutex::new(None),
            #[cfg(not(target_arch = "wasm32"))]
            logs: Mutex::new(Vec::new()),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn retain(&self, kaspad: Arc<dyn Kaspad + Send + Sync + 'static>) {
        self.kaspad.lock().unwrap().replace(kaspad);
    }

    pub fn create_rpc_client(config: &RpcConfig, network: Network) -> Result<Rpc> {
        match config {
            RpcConfig::Wrpc { url, encoding } => {
                // log_warning!("create_rpc_client - RPC URL: {:?}", url);
                let url = url.clone().unwrap_or_else(|| "127.0.0.1".to_string());
                let url =
                    KaspaRpcClient::parse_url(url, *encoding, NetworkId::from(network).into())?;

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

    pub fn wallet(&self) -> Arc<CoreWallet> {
        self.wallet.clone()
    }

    #[cfg(not(target_arch = "wasm32"))]
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

    pub fn rpc_url(&self) -> Option<String> {
        if !self.wallet().has_rpc() {
            None
        } else if let Ok(wrpc_client) = self
            .wallet()
            .rpc_api()
            .clone()
            .downcast_arc::<KaspaRpcClient>()
        {
            Some(wrpc_client.url().to_string())
        } else {
            None
        }
    }

    fn is_wrpc_client(&self) -> bool {
        self.wallet.has_rpc()
            && self
                .wallet()
                .rpc_api()
                .clone()
                .downcast_arc::<KaspaRpcClient>()
                .is_ok()
    }

    async fn disconnect_rpc(&self) -> Result<()> {
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

        Ok(())
    }

    pub async fn stop_all_services(&self) -> Result<()> {
        self.services_start_instant.lock().unwrap().take();

        if !self.wallet().has_rpc() {
            return Ok(());
        }

        let preemptive_disconnect = ENABLE_PREEMPTIVE_DISCONNECT && self.is_wrpc_client();

        if preemptive_disconnect {
            self.disconnect_rpc().await?;
        }

        for service in crate::runtime::runtime().services().into_iter() {
            let instant = Instant::now();
            service.clone().detach_rpc().await?;
            if instant.elapsed().as_millis() > 1_000 {
                log_warning!(
                    "WARNING: detach_rpc() for '{}' took {} msec",
                    service.name(),
                    instant.elapsed().as_millis()
                );
            }
        }

        if !preemptive_disconnect {
            self.disconnect_rpc().await?;
        }

        self.wallet().stop().await.expect("Unable to stop wallet");
        self.wallet().bind_rpc(None).await?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let kaspad = self.kaspad.lock().unwrap().take();
            if let Some(kaspad) = kaspad {
                if let Err(err) = kaspad.stop().await {
                    println!("error shutting down kaspad: {}", err);
                }
            }
        }

        Ok(())
    }

    pub async fn start_all_services(self: &Arc<Self>, rpc: Rpc, network: Network) -> Result<()> {
        self.services_start_instant
            .lock()
            .unwrap()
            .replace(Instant::now());

        *self.network.lock().unwrap() = network;

        let rpc_api = rpc.rpc_api().clone();

        self.wallet()
            .set_network_id(network.into())
            .expect("Can not change network id while the wallet is connected");

        self.wallet().bind_rpc(Some(rpc)).await.unwrap();
        self.wallet()
            .start()
            .await
            .expect("Unable to start wallet service");

        for service in crate::runtime::runtime().services().into_iter() {
            service.attach_rpc(&rpc_api).await?;
        }

        Ok(())
    }

    pub fn update_services(&self, node_settings: &NodeSettings, options: Option<RpcOptions>) {
        match KaspadServiceEvents::from_node_settings(node_settings, options) {
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

    fn network(&self) -> Network {
        *self.network.lock().unwrap()
    }

    async fn handle_network_change(&self, network: Network) -> Result<()> {
        if network != self.network() {
            self.application_events
                .send(Events::NetworkChange(network))
                .await?;
        }

        Ok(())
    }

    pub async fn connect_all_services(&self) -> Result<()> {
        for service in crate::runtime::runtime().services().into_iter() {
            service.connect_rpc().await?;
        }

        Ok(())
    }

    pub async fn disconnect_all_services(&self) -> Result<()> {
        for service in crate::runtime::runtime().services().into_iter() {
            service.disconnect_rpc().await?;
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn update_storage(&self) {
        const STORAGE_UPDATE_DELAY: Duration = Duration::from_millis(3000);

        let options = StorageUpdateOptions::default()
            .if_not_present()
            .with_delay(STORAGE_UPDATE_DELAY);

        runtime().update_storage(options);
    }
}

#[async_trait]
impl Service for KaspaService {
    fn name(&self) -> &'static str {
        "kaspa-service"
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        let wallet_events = self.wallet.multiplexer().channel();
        let _application_events_sender = self.application_events.sender.clone();

        loop {
            select! {

                msg = wallet_events.recv().fuse() => {
                    if let Ok(event) = msg {
                        use kaspa_wallet_core::events::Events as CoreWallet;

                        match *event {
                            CoreWallet::DAAScoreChange{ .. } => {
                            }
                            CoreWallet::Connect { .. } => {
                                self.connect_all_services().await?;
                            }
                            CoreWallet::Disconnect { .. } => {
                                self.disconnect_all_services().await?;
                            }
                            _ => {
                                // println!("wallet event: {:?}", event);
                            }
                        }
                        self.application_events.sender.send(crate::events::Events::Wallet{event}).await.unwrap();
                    } else {
                        break;
                    }
                }

                msg = self.as_ref().service_events.receiver.recv().fuse() => {

                    if let Ok(event) = msg {

                        match event {

                            #[cfg(not(target_arch = "wasm32"))]
                            KaspadServiceEvents::Stdout { line } => {

                                if !self.wallet().utxo_processor().is_synced() {
                                    self.wallet().utxo_processor().sync_proc().handle_stdout(&line).await?;
                                }

                                self.update_logs(line).await;
                            }

                            #[cfg(not(target_arch = "wasm32"))]
                            KaspadServiceEvents::StartInternalInProc { config, network } => {
                                self.stop_all_services().await?;

                                self.handle_network_change(network).await?;

                                let kaspad = Arc::new(inproc::InProc::default());
                                self.retain(kaspad.clone());
                                // this.kaspad.lock().unwrap().replace(kaspad.clone());

                                kaspad.clone().start(config).await.unwrap();

                                let rpc_api = kaspad.rpc_core_services().expect("Unable to obtain inproc rpc api");
                                let rpc_ctl = RpcCtl::new();
                                let rpc = Rpc::new(rpc_api, rpc_ctl.clone());

                                self.start_all_services(rpc, network).await?;
                                self.connect_rpc_client().await?;

                                self.update_storage();
                            },
                            #[cfg(not(target_arch = "wasm32"))]
                            KaspadServiceEvents::StartInternalAsDaemon { config, network } => {
                                self.stop_all_services().await?;

                                self.handle_network_change(network).await?;

                                let kaspad = Arc::new(daemon::Daemon::new(None, &self.service_events));
                                self.retain(kaspad.clone());
                                kaspad.clone().start(config).await.unwrap();

                                let rpc_config = RpcConfig::Wrpc {
                                    url : None,
                                    encoding : WrpcEncoding::Borsh,
                                };

                                let rpc = Self::create_rpc_client(&rpc_config, network).expect("Kaspad Service - unable to create wRPC client");
                                self.start_all_services(rpc, network).await?;
                                self.connect_rpc_client().await?;

                                self.update_storage();
                            },
                            #[cfg(not(target_arch = "wasm32"))]
                            KaspadServiceEvents::StartExternalAsDaemon { path, config, network } => {
                                self.stop_all_services().await?;

                                self.handle_network_change(network).await?;

                                let kaspad = Arc::new(daemon::Daemon::new(Some(path), &self.service_events));
                                self.retain(kaspad.clone());

                                kaspad.clone().start(config).await.unwrap();

                                let rpc_config = RpcConfig::Wrpc {
                                    url : None,
                                    encoding : WrpcEncoding::Borsh,
                                };

                                let rpc = Self::create_rpc_client(&rpc_config, network).expect("Kaspad Service - unable to create wRPC client");
                                self.start_all_services(rpc, network).await?;
                                self.connect_rpc_client().await?;

                                self.update_storage();
                            },
                            KaspadServiceEvents::StartRemoteConnection { rpc_config, network } => {
                                self.stop_all_services().await?;

                                self.handle_network_change(network).await?;

                                let rpc = Self::create_rpc_client(&rpc_config, network).expect("Kaspad Service - unable to create wRPC client");
                                self.start_all_services(rpc, network).await?;
                                self.connect_rpc_client().await?;
                            },

                            KaspadServiceEvents::Disable { network } => {
                                self.stop_all_services().await?;

                                self.handle_network_change(network).await?;

                                if self.wallet().is_open() {
                                    self.wallet().close().await.ok();
                                }

                                // re-apply network id to allow wallet
                                // to be opened offline in disconnected
                                // mode by changing network id in settings
                                self.wallet()
                                    .set_network_id(network.into()).ok();
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

        self.stop_all_services().await?;
        self.task_ctl.send(()).await.unwrap();

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

impl KaspadServiceEvents {
    pub fn from_node_settings(
        node_settings: &NodeSettings,
        options: Option<RpcOptions>,
    ) -> Result<Self> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {

                match &node_settings.node_kind {
                    KaspadNodeKind::Disable => {
                        Ok(KaspadServiceEvents::Disable { network : node_settings.network })
                    }
                    KaspadNodeKind::IntegratedInProc => {
                        // let config = ;
                        Ok(KaspadServiceEvents::StartInternalInProc { config : Config::from(node_settings.clone()), network : node_settings.network })
                    }
                    KaspadNodeKind::IntegratedAsDaemon => {
                        Ok(KaspadServiceEvents::StartInternalAsDaemon { config : Config::from(node_settings.clone()), network : node_settings.network })
                    }
                    KaspadNodeKind::ExternalAsDaemon => {
                        let path = node_settings.kaspad_daemon_binary.clone();
                        Ok(KaspadServiceEvents::StartExternalAsDaemon { path : PathBuf::from(path), config : Config::from(node_settings.clone()), network : node_settings.network })
                    }
                    KaspadNodeKind::Remote => {
                        Ok(KaspadServiceEvents::StartRemoteConnection { rpc_config : RpcConfig::from_node_settings(node_settings,options), network : node_settings.network })
                    }
                }

            } else {

                match &node_settings.node_kind {
                    KaspadNodeKind::Disable => {
                        Ok(KaspadServiceEvents::Disable { network : node_settings.network })
                    }
                    KaspadNodeKind::Remote => {
                        Ok(KaspadServiceEvents::StartRemoteConnection { rpc_config : RpcConfig::from_node_settings(node_settings,options), network : node_settings.network })
                    }
                }
            }
        }
    }
}
