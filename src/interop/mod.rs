use crate::imports::*;
mod wasm;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {

        pub mod signals;
        pub mod kaspad;
        pub use kaspad::KaspadService;

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
    runtime: Arc<AsyncRuntime>,
    executor: Arc<Executor>,
    wallet: Arc<WalletService>,
    application_events: channel::Channel<Events>,

    #[cfg(not(target_arch = "wasm32"))]
    kaspad: Arc<KaspadService>,
}
// #[derive(Default)]
#[derive(Clone)]
pub struct Interop {
    inner: Arc<Inner>,
}

impl Interop {
    pub fn new(ctx: &egui::Context, settings: &Settings) -> Self {
        let application_events = channel::Channel::unbounded(ctx);
        let runtime = Arc::new(AsyncRuntime::default());
        let executor = Arc::new(Executor::new(application_events.clone()));
        let wallet = Arc::new(WalletService::new(application_events.clone()));

        runtime.register(executor.clone());
        runtime.register(wallet.clone());

        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                let kaspad = Arc::new(KaspadService::new(application_events.clone(), settings));
                runtime.register(kaspad.clone());
            }
        }

        Self {
            inner: Arc::new(Inner {
                application_events,
                runtime,
                wallet,
                executor,
                #[cfg(not(target_arch = "wasm32"))]
                kaspad,
            }),
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

    pub fn wallet(&self) -> &Arc<runtime::Wallet> {
        &self.inner.wallet.wallet
    }

    pub fn executor_service(&self) -> &Arc<Executor> {
        &self.inner.executor
    }

    pub fn kaspad_service(&self) -> &Arc<KaspadService> {
        &self.inner.kaspad
    }

    pub fn application_events(&self) -> &Channel<Events> {
        &self.inner.application_events
    }

    pub async fn send(&self, msg: Events) -> Result<()> {
        self.inner.application_events.sender.send(msg).await?;
        Ok(())
    }

    pub fn try_send(&self, msg: Events) -> Result<()> {
        self.inner.application_events.sender.try_send(msg)?;
        Ok(())
    }

    // pub fn wallet_service(&self) -> &Arc<runtime::Wallet> {
    //     &self.inner.wallet.wallet
    // }
}
