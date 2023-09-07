use crate::imports::*;
mod wasm;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
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


pub struct Inner {
    runtime : Arc<AsyncRuntime>,
    wallet_service : Arc<WalletService>,
}
// #[derive(Default)]
#[derive(Clone)]
pub struct Interop {
    inner : Arc<Inner>
}

impl Interop {
    pub fn new() -> Self {
        let runtime = Arc::new(AsyncRuntime::default());
        let wallet = Arc::new(WalletService::new());
        runtime.register(wallet.clone());
        runtime.spawn();

        Self {
            inner : Arc::new(Inner { runtime, wallet_service: wallet })
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

    pub fn wallet(&self) -> &Arc<runtime::Wallet> {
        &self.inner.wallet_service.wallet
    }

}