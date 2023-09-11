use crate::imports::*;
// use std::{
//     pin::Pin,
//     sync::atomic::{AtomicBool, Ordering},
// };

// mod wasm;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {

        pub mod signals;
        // pub mod kaspad;
        // pub use kaspad::KaspadService;

        // mod tokio;
        // pub use tokio::*;

    } else {
        // pub use wasm::*;
    }
}

pub mod service;
use futures_util::future::join_all;
pub use service::Service;

// pub mod wallet;
// pub use wallet::WalletService;

// pub mod executor;
// pub use executor::Executor;

pub mod kaspa;
pub use kaspa::KaspaService;

pub mod channel;
pub use channel::Channel;

pub mod payload;
pub use payload::Payload;

pub struct Inner {
    // runtime: Arc<AsyncRuntime>,
    // executor: Arc<Executor>,
    // wallet: Arc<WalletService>,
    application_events: channel::Channel<Events>,

    // #[cfg(not(target_arch = "wasm32"))]
    kaspa: Arc<KaspaService>,

    services: Mutex<Vec<Arc<dyn Service + Send + Sync + 'static>>>,
    // egui_ctx : egui::Context,
}
// #[derive(Default)]
#[derive(Clone)]
pub struct Interop {
    inner: Arc<Inner>,
}

impl Interop {
    pub fn new(egui_ctx: &egui::Context, settings: &Settings) -> Self {
        let application_events = channel::Channel::unbounded(egui_ctx.clone());
        // let runtime = Arc::new(AsyncRuntime::default());
        // let executor = Arc::new(Executor::new(application_events.clone()));
        // let wallet = Arc::new(WalletService::new(application_events.clone()));

        // runtime.register(executor.clone());
        // runtime.register(wallet.clone());
        let kaspa = Arc::new(KaspaService::new(application_events.clone(), settings));

        let services: Vec<Arc<dyn Service + Send + Sync + 'static>> = vec![
            kaspa.clone(),
            // wallet.clone(),
        ];
        // cfg_if! {
        //     if #[cfg(not(target_arch = "wasm32"))] {
        //         runtime.register(kaspad.clone());
        //     }
        // }

        let interop = Self {
            inner: Arc::new(Inner {
                application_events,
                kaspa,
                services: Mutex::new(services),
                // egui_ctx : egui_ctx.clone(),
                // runtime,
                // wallet,
                // executor,
                // #[cfg(not(target_arch = "wasm32"))]
                // kaspad,
            }),
        };

        register(Some(interop.clone()));

        interop
    }

    pub fn run(&self) {
        // self.inner.runtime.spawn();

        // register(Some(self));

        let services = self.services();
        for service in services {
            spawn(async move { service.spawn().await });
        }
        // let futures = services.into_iter().map(|service|service.spawn()).collect::<Vec<_>>();
        // spawn(async move {
        //     try_join_all(futures).await.unwrap();
        // });
    }

    pub fn services(&self) -> Vec<Arc<dyn Service + Send + Sync + 'static>> {
        self.inner.services.lock().unwrap().clone()
    }

    pub fn shutdown(&self) {
        // self.inner.runtime.shutdown();
        self.services()
            .into_iter()
            .for_each(|service| service.terminate());
    }

    pub async fn join(&self) {
        let futures = self
            .services()
            .into_iter()
            .map(|service| service.join())
            .collect::<Vec<_>>();
        join_all(futures).await;
    }

    pub fn drop(&self) {
        register(None);
    }
    // cfg_if! {

    //     if #[cfg(not(target_arch = "wasm32"))] {
    //         pub fn join(&self) {
    //             self.inner.runtime.join();
    //         }
    //     } else {
    //         pub async fn join(&self) {
    //             self.inner.runtime.join().await;
    //         }
    //     }
    // }

    // pub fn wallet_service(&self) -> &Arc<WalletService> {
    //     &self.inner.wallet
    // }

    pub fn wallet(&self) -> &Arc<runtime::Wallet> {
        &self.inner.kaspa.wallet
    }

    // pub fn executor_service(&self) -> &Arc<Executor> {
    //     &self.inner.executor
    // }

    // pub fn kaspad_service(&self) -> &Arc<KaspadService> {
    //     &self.inner.kaspad
    // }

    pub fn application_events(&self) -> &Channel<Events> {
        &self.inner.application_events
    }

    pub async fn send(&self, msg: Events) -> Result<()> {
        self.inner.application_events.sender.send(msg).await?;
        Ok(())
    }

    pub fn try_send(&self, msg: Events) -> Result<()> {
        println!("interop try_send()");
        self.inner.application_events.sender.try_send(msg)?;
        Ok(())
    }

    // pub fn wallet_service(&self) -> &Arc<runtime::Wallet> {
    //     &self.inner.wallet.wallet
    // }

    pub fn spawn_task<F>(&self, task: F)
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let sender = self.inner.application_events.sender.clone();
        workflow_core::task::spawn(async move {
            if let Err(err) = task.await {
                sender
                    .send(Events::Error(Box::new(err.to_string())))
                    .await
                    .unwrap();
                println!("spawned task error: {:?}", err);
            }
        });
    }

    pub fn spawn_task_with_result<R, F>(
        &self,
        semaphore: &Payload<std::result::Result<R, Error>>,
        task: F,
    ) where
        R: Clone + Send + 'static,
        F: Future<Output = Result<R>> + Send + 'static,
    {
        // let sender = self.inner.application_events.sender.clone();
        let semaphore = (*semaphore).clone();
        workflow_core::task::spawn(async move {
            let result = task.await;
            // semaphore.set(result);
            match result {
                Ok(r) => semaphore.store(Ok(r)),
                Err(err) => {
                    semaphore.store(Err(err));
                    // sender.send(Events::Error(Box::new(err.to_string()))).await.unwrap();
                    // println!("spawned task error: {:?}", err);
                }
            }
            // if let Err(err) = task.await {
            //     sender.send(Events::Error(Box::new(err.to_string()))).await.unwrap();
            //     println!("spawned task error: {:?}", err);
            // }
        });
    }
}

// impl Drop for Interop {
//     fn drop(&mut self) {
//         register(None);
//     }
// }

static mut INTEROP: Option<Interop> = None;

fn interop() -> &'static Interop {
    unsafe {
        if let Some(interop) = &INTEROP {
            interop
        } else {
            panic!("interop not initialized")
        }
    }
}

fn register(interop: Option<Interop>) {
    unsafe {
        INTEROP = interop;
    }
}

pub fn spawn<F>(task: F)
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    interop().spawn_task(task);
}

pub fn spawn_with_result<R, F>(semaphore: &Payload<std::result::Result<R, Error>>, task: F)
where
    R: Clone + Send + 'static,
    F: Future<Output = Result<R>> + Send + 'static,
{
    interop().spawn_task_with_result(semaphore, task);
}
