use std::time::Duration;

use crate::imports::*;
use crate::runtime::Service;
pub use futures::{future::FutureExt, select, Future};
#[allow(unused_imports)]
use kaspa_wallet_core::rpc::{
    ConnectOptions, ConnectStrategy, NotificationMode, Rpc, RpcCtl, WrpcEncoding,
};

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
            // async fn halt(&self) -> Result<()>;
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
            match KaspadServiceEvents::try_from(&settings.node) {
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

    pub async fn stop_all_services(&self) -> Result<()> {
        if !self.wallet().has_rpc() {
            return Ok(());
        }

        for service in crate::runtime::runtime().services().into_iter() {
            service.detach_rpc().await?;
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
                if let Err(err) = kaspad.stop().await {
                    println!("error shutting down kaspad: {}", err);
                }
            }
        }

        Ok(())
    }

    pub async fn start_all_services(self: &Arc<Self>, rpc: Rpc, network: Network) -> Result<()> {
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
}

#[async_trait]
impl Service for KaspaService {
    fn name(&self) -> &'static str {
        "kaspa-service"
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        let this = self.clone();
        let wallet_events = this.wallet.multiplexer().channel();
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
                                this.connect_all_services().await?;
                            }
                            CoreWallet::Disconnect { .. } => {
                                this.disconnect_all_services().await?;
                            }
                            _ => {
                                println!("wallet event: {:?}", event);
                            }
                        }
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

                                let kaspad = Arc::new(inproc::InProc::default());
                                this.retain(kaspad.clone());
                                // this.kaspad.lock().unwrap().replace(kaspad.clone());

                                kaspad.clone().start(config).await.unwrap();

                                let rpc_api = kaspad.rpc_core_services().expect("Unable to obtain inproc rpc api");
                                let rpc_ctl = RpcCtl::new();
                                let rpc = Rpc::new(rpc_api, rpc_ctl.clone());

                                this.start_all_services(rpc, network).await?;
                                this.connect_rpc_client().await?;

                            },
                            #[cfg(not(target_arch = "wasm32"))]
                            KaspadServiceEvents::StartInternalAsDaemon { config, network } => {
                                self.stop_all_services().await?;

                                let kaspad = Arc::new(daemon::Daemon::new(None, &this.service_events));
                                this.retain(kaspad.clone());
                                kaspad.clone().start(config).await.unwrap();

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

                                let kaspad = Arc::new(daemon::Daemon::new(Some(path), &this.service_events));
                                this.retain(kaspad.clone());

                                kaspad.clone().start(config).await.unwrap();

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

                                let rpc = Self::create_rpc_client(&rpc_config, network).expect("Kaspad Service - unable to create wRPC client");
                                this.start_all_services(rpc, network).await?;
                                this.connect_rpc_client().await?;
                            },

                            KaspadServiceEvents::Disable { network } => {
                                self.stop_all_services().await?;

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
                        Ok(KaspadServiceEvents::StartRemoteConnection { rpc_config : node_settings.into(), network : node_settings.network })
                    }
                }

            } else {

                match &node_settings.node_kind {
                    KaspadNodeKind::Disable => {
                        Ok(KaspadServiceEvents::Disable { network : node_settings.network })
                    }
                    KaspadNodeKind::Remote => {
                        Ok(KaspadServiceEvents::StartRemoteConnection { rpc_config : node_settings.into(), network : node_settings.network })
                    }
                }
            }
        }
    }
}
