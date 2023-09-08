pub mod config;
pub mod daemon;
pub mod inproc;
pub use config::Config;

use crate::imports::*;
use crate::interop::AsyncService;
pub use futures::{future::FutureExt, select, Future};
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
    Exit,
}

pub struct KaspadService {
    pub application_events: interop::Channel<Events>,
    pub service_events: Channel<KaspadServiceEvents>,
    pub network: Mutex<Network>,
    pub kaspad: Mutex<Option<Arc<dyn Kaspad + Send + Sync + 'static>>>, // pub shutdown : AtomicBool,
                                                                        // pub wallet : Arc<runtime::Wallet>,
}

impl KaspadService {
    pub fn new(application_events: interop::Channel<Events>, settings: &Settings) -> Self {
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
            _ => {}
        }

        Self {
            application_events,
            service_events,
            network: Mutex::new(settings.network),
            kaspad: Mutex::new(None),
            // kaspad : Arc::
            // config : Mutex::new(),
        }
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

                                    if let Some(kaspad) = self.kaspad.lock().unwrap().take() {
                                        if let Err(err) = kaspad.stop() {
                                            println!("error stopping kaspad: {}", err);
                                        }
                                        // TODO - handle RPC bindings...
                                    }

                                    let kaspad = Arc::new(inproc::InProc::default());
                                    self.kaspad.lock().unwrap().replace(kaspad.clone());
                                    kaspad.start(config.into()).unwrap();

                                },
                                KaspadServiceEvents::StartInternalAsDaemon { config:_ } => {

                                },
                                KaspadServiceEvents::StartExternalAsDaemon { path:_, config:_ } => {

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

            println!("shutting down kaspad manager service...");
            if let Some(kaspad) = self.kaspad.lock().unwrap().take() {
                if let Err(err) = kaspad.stop() {
                    println!("error stopping kaspad: {}", err);
                }

                // TODO - handle RPC bindings...
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
