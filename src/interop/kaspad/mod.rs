pub mod config;
pub mod daemon;
pub mod inproc;
pub use config::Config;

use crate::imports::*;
use crate::interop::{AsyncService, WalletService};
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

pub struct KaspadService {
    pub application_events: interop::Channel<Events>,
    pub service_events: Channel<KaspadServiceEvents>,
    pub network: Mutex<Network>,
    pub kaspad: Mutex<Option<Arc<dyn Kaspad + Send + Sync + 'static>>>,
    pub wallet_service: Arc<WalletService>,
}

impl KaspadService {
    pub fn new(
        application_events: interop::Channel<Events>,
        settings: &Settings,
        wallet_service: Arc<WalletService>,
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

        Self {
            application_events,
            service_events,
            network: Mutex::new(settings.network),
            kaspad: Mutex::new(None),
            wallet_service,
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
}

// impl Drop for Executor {
//     fn drop(&mut self) {
//     }
// }

impl AsyncService for KaspadService {
    fn start(self: Arc<Self>) -> BoxFuture<'static, Result<()>> {
        println!("kaspad manager service starting...");
        let this = self.clone();
        let _application_events_sender = self.application_events.sender.clone();
        Box::pin(async move {
            loop {
                select! {
                    msg = this.as_ref().service_events.receiver.recv().fuse() => {

                        if let Ok(event) = msg {
                            match event {

                                KaspadServiceEvents::StartInternalInProc { config } => {

                                    this.wallet_service.wallet().stop().await.expect("Unable to stop wallet");
                                    this.wallet_service.wallet().bind_rpc(None).await?;

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

                                    this.wallet_service.wallet().bind_rpc(Some(rpc)).await.unwrap();
                                    this.wallet_service.wallet().start().await.expect("Unable to stop wallet");


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
            this.wallet_service
                .wallet()
                .stop()
                .await
                .expect("Unable to stop wallet");
            this.wallet_service.wallet().bind_rpc(None).await?;

            println!("shutting down kaspad manager service...");
            if let Some(kaspad) = self.kaspad.lock().unwrap().take() {
                if let Err(err) = kaspad.stop() {
                    println!("error stopping kaspad: {}", err);
                }
            }

            Ok(())
        })
    }

    fn signal_exit(self: Arc<Self>) {
        self.service_events
            .sender
            .try_send(KaspadServiceEvents::Exit)
            .unwrap();
    }

    fn stop(self: Arc<Self>) -> BoxFuture<'static, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
}
