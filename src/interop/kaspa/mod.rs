pub mod config;
pub use config::Config;
use workflow_core::prelude::DuplexChannel;
pub mod daemon;
// pub use daemon::Daemon;
pub mod inproc;
// pub use inproc::InProc;


use crate::imports::*;
use crate::interop::Service;
pub use futures::{future::FutureExt, select, Future};
use kaspa_wallet_core::rpc::{NotificationMode, Rpc, RpcCtl, WrpcEncoding};
pub use kaspad::args::Args;
use std::path::PathBuf;

pub trait Kaspad {
    fn start(&self, args: Args) -> Result<()>;
    fn stop(&self) -> Result<()>;
}

// ---------

pub enum KaspadServiceEvents {
    StartInternalInProc { config: Config },
    StartInternalAsDaemon { config: Config },
    StartExternalAsDaemon { path: PathBuf, config: Config },
    StartRemoteConnection { url: String },
    Exit,
}

pub struct KaspaService {
    pub application_events: interop::Channel<Events>,
    pub service_events: Channel<KaspadServiceEvents>,
    pub task_ctl: Channel<()>,//DuplexChannel<()>,
    pub network: Mutex<Network>,
    pub kaspad: Mutex<Option<Arc<dyn Kaspad + Send + Sync + 'static>>>,
    // pub wallet_service: Arc<WalletService>,
    pub wallet: Arc<runtime::Wallet>,

}

impl KaspaService {
    pub fn new(
        application_events: interop::Channel<Events>,
        settings: &Settings,
        // wallet_service: Arc<WalletService>,
    ) -> Self {
        let service_events = Channel::unbounded();

        let config = Config::from(settings.clone());

        // if settings.kaspad_node =
        match &settings.kaspad_node {
            KaspadNodeKind::InternalInProc => {
                service_events
                    .sender
                    .try_send(KaspadServiceEvents::StartInternalInProc { config })
                    .unwrap();
            }
            KaspadNodeKind::InternalAsDaemon => {
                service_events
                    .sender
                    .try_send(KaspadServiceEvents::StartInternalAsDaemon { config })
                    .unwrap();
            }
            KaspadNodeKind::ExternalAsDaemon { path } => {
                let path = PathBuf::from(path);
                service_events
                    .sender
                    .try_send(KaspadServiceEvents::StartExternalAsDaemon { path, config })
                    .unwrap();
            }
            KaspadNodeKind::Remote { url } => {
                service_events
                    .sender
                    .try_send(KaspadServiceEvents::StartRemoteConnection { url: url.clone() })
                    .unwrap();
            }
        }

        let storage = runtime::Wallet::local_store().unwrap_or_else(|e| {
            panic!("Failed to open local store: {}", e);
        });

        let wallet = runtime::Wallet::try_new(storage, None).unwrap_or_else(|e| {
            panic!("Failed to create wallet instance: {}", e);
        });

        Self {
            application_events,
            service_events,
            task_ctl: Channel::oneshot(),
            network: Mutex::new(settings.network),
            kaspad: Mutex::new(None),
            wallet: Arc::new(wallet),

            // wallet_service,
            // kaspad : Arc::
            // config : Mutex::new(),
        }
    }

    pub fn create_wrpc_client(&self) -> Result<Rpc> {
        let rpc_client = Arc::new(KaspaRpcClient::new_with_args(
            WrpcEncoding::Borsh,
            NotificationMode::MultiListeners,
            "wrpc://127.0.0.1:17110",
        )?);
        let rpc_ctl = rpc_client.ctl().clone();
        let rpc_api: Arc<DynRpcApi> = rpc_client;
        Ok(Rpc::new(rpc_api, rpc_ctl))
    }
    // pub fn shutdown(&self) {
    //     self.shutdown.store(true, Ordering::SeqCst);
    // }

    pub fn wallet(&self) -> &Arc<runtime::Wallet> {
        &self.wallet
    }


}

// impl Drop for Executor {
//     fn drop(&mut self) {
//     }
// }

#[async_trait]
impl Service for KaspaService {
    async fn spawn(self: Arc<Self>) -> Result<()> {
        println!("kaspad manager service starting...");
        let this = self.clone();

        println!("starting wallet...");
        this.wallet.start().await.unwrap_or_else(|err| {
            println!("Wallet start error: {:?}", err);
        });


        let wallet_events = this.wallet.multiplexer().channel();

        let _application_events_sender = self.application_events.sender.clone();
        // spawn(async move {
            loop {
                println!("loop...");
                select! {

                    msg = wallet_events.recv().fuse() => {
                        if let Ok(event) = msg {
                            println!("wallet event: {:?}", event);
                            this.application_events.sender.send(crate::events::Events::Wallet{event}).await.unwrap();
                        } else {
                            break;
                        }
                    }

                    msg = this.as_ref().service_events.receiver.recv().fuse() => {

                        if let Ok(event) = msg {
                            match event {

                                KaspadServiceEvents::StartInternalInProc { config } => {

                                    this.wallet().stop().await.expect("Unable to stop wallet");
                                    this.wallet().bind_rpc(None).await?;

                                    if let Some(kaspad) = self.kaspad.lock().unwrap().take() {
                                        println!("*** STOPPING KASPAD ***");
                                        if let Err(err) = kaspad.stop() {
                                            println!("error stopping kaspad: {}", err);
                                        }
                                    }

                                    println!("*** STARTING NEW KASPAD ***");
                                    let kaspad = Arc::new(inproc::InProc::default());
                                    this.kaspad.lock().unwrap().replace(kaspad.clone());
                                    kaspad.start(config.into()).unwrap();


                                    println!("*** SETTING UP DIRECT RPC BINDING ***");
                                    let rpc_api = kaspad.rpc_core_services().expect("Unable to obtain inproc rpc api");
                                    let rpc_ctl = RpcCtl::new();
                                    let rpc = Rpc::new(rpc_api, rpc_ctl.clone());


                                    // - CONNECT NEVER REACHES BECAUSE WHEN IT IS BROADCASTED,
                                    // - MULTIPLEXER CLIENT DOES NOT YET EXISTS

                                    this.wallet().bind_rpc(Some(rpc)).await.unwrap();
                                    this.wallet().start().await.expect("Unable to stop wallet");


                                    rpc_ctl.try_signal_open().expect("Unable to signal `open` event to rpc ctl");

                                },
                                KaspadServiceEvents::StartInternalAsDaemon { config:_ } => {



                                },
                                KaspadServiceEvents::StartExternalAsDaemon { path:_, config:_ } => {

                                },
                                KaspadServiceEvents::StartRemoteConnection { url: _ } => {

                                },

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

            println!("stopping wallet from node manager...");
            this.wallet()
                .stop()
                .await
                .expect("Unable to stop wallet");
            this.wallet().bind_rpc(None).await?;

            println!("shutting down kaspad manager service...");
            if let Some(kaspad) = self.kaspad.lock().unwrap().take() {
                if let Err(err) = kaspad.stop() {
                    println!("error stopping kaspad: {}", err);
                }
            }

            this.task_ctl.send(()).await.unwrap();
            // Ok(())
        // });

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
        // Box::pin(async move { Ok(()) })
    }
}
