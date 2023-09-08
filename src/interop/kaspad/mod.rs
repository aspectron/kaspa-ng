pub mod inproc;
pub mod daemon;

use std::path::PathBuf;

use crate::imports::*;
pub use futures::{future::FutureExt, select, Future};
pub use kaspad::args::Args;
use crate::interop::AsyncService;

pub trait Kaspad {
    fn start(&self, args: Args) -> Result<()>;
    fn stop(&self) -> Result<()>;
}

// ---------




pub enum KaspadServiceEvents {
    StartInternalInProc { args : Args },
    StartInternalAsDaemon { args : Args },
    StartExternalAsDaemon { path : PathBuf, args : Args },
    Exit,
}


pub struct KaspadService {
    pub application_events : Channel<Events>,
    pub executor_events : Channel<KaspadServiceEvents>,
    // pub shutdown : AtomicBool,
    // pub wallet : Arc<runtime::Wallet>,
}

impl KaspadService {
    pub fn new(application_events : Channel<crate::events::Events>) -> Self {

        Self {
            application_events,
            executor_events : Channel::unbounded(),
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

        println!("kaspad manager starting...");
        let this = self.clone();
        let application_events_sender = self.application_events.sender.clone();
        Box::pin(async move {

            // println!("starting wallet...");
            // this.wallet.start().await.unwrap_or_else(|err| {
            //     println!("Wallet start error: {:?}", err);
            // });

            loop {
                select! {
                    msg = this.as_ref().executor_events.receiver.recv().fuse() => {
                        // println!("Wallet received message: {:?}", msg);

                        if let Ok(event) = msg {
                            match event {

                                KaspadServiceEvents::StartInternalInProc { args } => {

                                },
                                KaspadServiceEvents::StartInternalAsDaemon { args } => {

                                },
                                KaspadServiceEvents::StartExternalAsDaemon { path, args } => {

                                },

                                // ExecutorEvents::Spawn(task) => {
                                //     let sender = application_events_sender.clone();
                                //     workflow_core::task::spawn(async move {
                                //         if let Err(err) = task.await {
                                //             sender.send(Events::Error(err.to_string())).await.unwrap();
                                //             println!("spawned task error: {:?}", err);
                                //         }
                                //     });
                                // },
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

            Ok(())
        })
    }
    
    fn signal_exit(self: Arc<Self>) {
        self.executor_events.sender.try_send(KaspadServiceEvents::Exit).unwrap();
    }

    fn stop(self: Arc<Self>) -> BoxFuture<'static, Result<()>> {
        Box::pin(async move {

            Ok(())
        })

    }
}
