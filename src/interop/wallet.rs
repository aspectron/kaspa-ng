use crate::imports::*;
pub use futures::{future::FutureExt, select, Future};

use crate::interop::AsyncService;

#[derive(Debug)]
pub enum Events {
    Exit,
}

pub struct WalletService {
    pub application_events: interop::Channel<crate::events::Events>,
    pub service_events: Channel<Events>,
    pub wallet: Arc<runtime::Wallet>,
}

impl WalletService {
    pub fn new(application_events: interop::Channel<crate::events::Events>) -> Self {
        let storage = runtime::Wallet::local_store().unwrap_or_else(|e| {
            panic!("Failed to open local store: {}", e);
        });

        let wallet = runtime::Wallet::try_new(storage, None).unwrap_or_else(|e| {
            panic!("Failed to create wallet instance: {}", e);
        });

        Self {
            application_events,
            service_events: Channel::unbounded(),
            wallet: Arc::new(wallet),
        }
    }

    pub fn wallet(&self) -> &Arc<runtime::Wallet> {
        &self.wallet
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

            let wallet_multiplexer = this.wallet.multiplexer().channel();

            loop {
                select! {
                    msg = this.as_ref().service_events.receiver.recv().fuse() => {
                        if let Ok(event) = msg {
                            match event {
                                Events::Exit => {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    },
                    msg = wallet_multiplexer.recv().fuse() => {
                        if let Ok(event) = msg {
                            println!("wallet event: {:?}", event);
                            this.application_events.sender.send(crate::events::Events::Wallet{event}).await.unwrap();
                        } else {
                            break;
                        }
                    }
                }
            }

            // self.wallet.stop().await?;

            Ok(())
        })
    }

    fn signal_exit(self: Arc<Self>) {
        self.service_events.sender.try_send(Events::Exit).unwrap();
    }

    fn stop(self: Arc<Self>) -> BoxFuture<'static, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
}
