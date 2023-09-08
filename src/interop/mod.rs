use crate::imports::*;
mod wasm;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        
        pub mod signals;
        pub mod kaspad;

        mod tokio;
        pub use tokio::*;
    } else {
        pub use wasm::*;
    }
}

pub mod service;
pub use service::AsyncService;

pub mod wallet;
pub use wallet::WalletService;

pub mod executor;
pub use executor::Executor;

pub mod channel;
pub use channel::Channel;

pub struct Inner {
    runtime : Arc<AsyncRuntime>,
    wallet : Arc<WalletService>,
    executor : Arc<Executor>,
    channel : channel::Channel<Events>,
}
// #[derive(Default)]
#[derive(Clone)]
pub struct Interop {
    inner : Arc<Inner>
}

impl Interop {
    pub fn new(ctx : &egui::Context) -> Self {
        let events = channel::Channel::unbounded(ctx);
        let runtime = Arc::new(AsyncRuntime::default());
        let executor = Arc::new(Executor::new(events.clone()));
        let wallet = Arc::new(WalletService::new());
        runtime.register(wallet.clone());
        runtime.register(executor.clone());
        // runtime.spawn();

        Self {
            inner : Arc::new(Inner { channel: events, runtime, wallet, executor })
        }
    }

    pub fn spawn(&self) {

        self.inner.runtime.spawn();
    }

    pub fn shutdown(&self) {
        self.inner.runtime.shutdown();
    }

    cfg_if! {
        if #[cfg(not(target_arch = "wasm32"))] {
            pub fn join(&self) {
                self.inner.runtime.join();
            }
        } else {
            pub async fn join(&self) {
                self.inner.runtime.join().await;
            }
        }
    }

    pub fn wallet_service(&self) -> &Arc<WalletService> {
        &self.inner.wallet
    }

    pub fn executor_service(&self) -> &Arc<Executor> {
        &self.inner.executor
    }

    pub fn channel(&self) -> &Channel<Events> {
        &self.inner.channel
    }

    pub fn try_send(&self, msg: Events) -> Result<()> {
        self.inner.channel.sender.try_send(msg)?;
        Ok(())
    }

    // pub fn wallet_service(&self) -> &Arc<runtime::Wallet> {
    //     &self.inner.wallet.wallet
    // }

}