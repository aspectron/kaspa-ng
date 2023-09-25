// pub mod service;

use crate::imports::*;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        // pub mod signals;
        pub mod panic;
    } else {
        // ...
    }
}

pub mod service;
use futures_util::future::join_all;
pub use service::Service;

pub mod kaspa;
pub use kaspa::KaspaService;

// pub mod channel;
// pub use channel::Channel;

// pub mod payload;
// pub use payload::Payload;

pub struct Inner {
    // application_events: channel::Channel<Events>,
    // kaspa: Mutex<Arc<KaspaService>,
    services: Mutex<Vec<Arc<dyn Service + Send + Sync + 'static>>>,
}

#[derive(Clone)]
pub struct Runtime {
    inner: Arc<Inner>,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime::new(&[])
        // let services: Vec<Arc<dyn Service + Send + Sync + 'static>> = vec![];

        // let runtime = Self {
        //     inner: Arc::new(Inner {
        //         // application_events,
        //         // kaspa,
        //         services: Mutex::new(services),
        //     }),
        // };

        // register_global(Some(runtime.clone()));

        // runtime
    }
}

impl Runtime {

    pub fn new(services : &[Arc<dyn Service + Send + Sync + 'static>]) -> Self {
        // let runtime = Self::default();
        // services.iter().for_each(|service| runtime.register_service(service.clone()));
        // runtime


        // let services: Vec<Arc<dyn Service + Send + Sync + 'static>> = vec![];

        let runtime = Self {
            inner: Arc::new(Inner {
                // application_events,
                // kaspa,
                services: Mutex::new(services.to_vec()),
            }),
        };

        register_global(Some(runtime.clone()));

        runtime
    }

    pub fn register_service(&self, service: Arc<dyn Service + Send + Sync + 'static>) {
        self.inner.services.lock().unwrap().push(service);
    }

    pub fn start(&self) {
        let services = self.services();
        for service in services {
            spawn(async move { service.spawn().await });
        }
    }

    pub fn services(&self) -> Vec<Arc<dyn Service + Send + Sync + 'static>> {
        self.inner.services.lock().unwrap().clone()
    }

    pub fn shutdown(&self) {
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
        register_global(None);
    }

}

static mut RUNTIME: Option<Runtime> = None;

fn _runtime() -> &'static Runtime {
    unsafe {
        if let Some(runtime) = &RUNTIME {
            runtime
        } else {
            panic!("runtime not initialized")
        }
    }
}

fn register_global(runtime: Option<Runtime>) {
    unsafe {
        RUNTIME = runtime;
    }
}
