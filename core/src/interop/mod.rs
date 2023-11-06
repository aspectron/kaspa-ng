use crate::imports::*;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        pub mod signals;
        pub mod panic;
    } else {
        // ...
    }
}

use crate::runtime::KaspaService;
use crate::runtime::Runtime;

// pub mod channel;
// use crate::channel::Channel;

pub mod payload;
pub use payload::Payload;

pub struct Inner {
    application_events: ApplicationEventsChannel,
    kaspa: Arc<KaspaService>,
    runtime: Runtime,
}

#[derive(Clone)]
pub struct Interop {
    inner: Arc<Inner>,
}

impl Interop {
    pub fn new(egui_ctx: &egui::Context, settings: &Settings) -> Self {
        let application_events = ApplicationEventsChannel::unbounded(Some(egui_ctx.clone()));
        let kaspa = Arc::new(KaspaService::new(application_events.clone(), settings));

        let runtime = Runtime::new(&[kaspa.clone()]);
        // runtime.register_service(kaspa.clone());

        let interop = Self {
            inner: Arc::new(Inner {
                application_events,
                kaspa,
                runtime,
                // services: Mutex::new(services),
            }),
        };

        register_global(Some(interop.clone()));

        interop
    }

    pub fn runtime(&self) -> &Runtime {
        &self.inner.runtime
    }

    pub fn start(&self) {
        self.runtime().start();
        // let services = self.services();
        // for service in services {
        //     spawn(async move { service.spawn().await });
        // }
    }

    // pub fn services(&self) -> Vec<Arc<dyn Service + Send + Sync + 'static>> {
    //     self.inner.services.lock().unwrap().clone()
    // }

    pub fn shutdown(&self) {
        self.runtime().shutdown();
        // self.services()
        //     .into_iter()
        //     .for_each(|service| service.terminate());
    }

    pub async fn join(&self) {
        self.runtime().join().await;
        // let futures = self
        //     .services()
        //     .into_iter()
        //     .map(|service| service.join())
        //     .collect::<Vec<_>>();
        // join_all(futures).await;
    }

    pub fn drop(&self) {
        register_global(None);
    }

    // pub fn wallet(&self) -> &Arc<runtime::Wallet> {
    //     &self.inner.kaspa.wallet
    // }
    pub fn wallet(&self) -> Arc<dyn WalletApi> {
        self.inner.kaspa.wallet()
    }

    pub fn kaspa_service(&self) -> &Arc<KaspaService> {
        &self.inner.kaspa
    }

    pub fn application_events(&self) -> &ApplicationEventsChannel {
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
        payload: &Payload<std::result::Result<R, Error>>,
        task: F,
    ) where
        R: Clone + Send + 'static,
        F: Future<Output = Result<R>> + Send + 'static,
    {
        let payload = (*payload).clone();
        payload.mark_pending();
        workflow_core::task::spawn(async move {
            let result = task.await;
            match result {
                Ok(r) => payload.store(Ok(r)),
                Err(err) => {
                    payload.store(Err(err));
                }
            }
        });
    }
}

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

fn register_global(interop: Option<Interop>) {
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

pub fn spawn_with_result<R, F>(payload: &Payload<std::result::Result<R, Error>>, task: F)
where
    R: Clone + Send + 'static,
    F: Future<Output = Result<R>> + Send + 'static,
{
    interop().spawn_task_with_result(payload, task);
}
