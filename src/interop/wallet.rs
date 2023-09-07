
use crate::imports::*;
pub use futures::{future::FutureExt, select, Future};
use std::sync::atomic::{AtomicBool,Ordering};

use crate::interop::AsyncService;

#[derive(Debug)]
pub enum Events {
    Noop,
    Open { name : Option<String>, secret : Secret },
    Exit,
}

pub struct WalletService {
    pub channel : Channel<Events>,
    pub shutdown : AtomicBool,
    pub wallet : Arc<runtime::Wallet>,
}

impl WalletService {
    pub fn new() -> Self {

        let storage = runtime::Wallet::local_store().unwrap_or_else(|e| {
            panic!("Failed to open local store: {}", e);
        });


        let wallet = runtime::Wallet::try_new(storage, None).unwrap_or_else(|e| {
            panic!("Failed to create wallet instance: {}", e);
        });


        Self {
            channel : Channel::unbounded(),
            shutdown : AtomicBool::new(false),
            wallet : Arc::new(wallet),
        }
    }

    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}

impl AsyncService for WalletService {
    fn start(self: Arc<Self>) -> BoxFuture<'static, Result<()>> {

        let this = self.clone();
        Box::pin(async move {

            println!("starting wallet...");
            this.wallet.start().await.unwrap_or_else(|err| {
                println!("Wallet start error: {:?}", err);
            });

            loop {
                select! {
                    msg = this.as_ref().channel.receiver.recv().fuse() => {
                        println!("Wallet received message: {:?}", msg);

                        if let Ok(event) = msg {
                            self.handle_event(event).await.unwrap_or_else(|err| {
                                println!("Wallet service error: {:?}", err);
                            });

                            if self.shutdown.load(Ordering::SeqCst) {
                                break;
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
        self.channel.sender.try_send(Events::Exit).unwrap();
    }

    fn stop(self: Arc<Self>) -> BoxFuture<'static, Result<()>> {
        Box::pin(async move {

            Ok(())
        })

    }
}

impl WalletService {
    async fn handle_event(self : &Arc<Self>, event : Events) -> Result<()> {

        match event {
            Events::Noop => {},
            Events::Exit => {
                println!("stopping wallet...");
                self.wallet.stop().await?;
                self.shutdown.store(true, Ordering::SeqCst);
            }
            Events::Open { name, secret } => {
                self.wallet.load(secret.0, name).await?;
            }
        }

        Ok(())
    }
}