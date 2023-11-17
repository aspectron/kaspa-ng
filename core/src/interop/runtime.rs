use crate::imports::*;

#[async_trait]
pub trait Service: Sync + Send {
    async fn spawn(self: Arc<Self>) -> Result<()>;
    async fn join(self: Arc<Self>) -> Result<()>;
    fn terminate(self: Arc<Self>);
}

pub struct Inner {
    services: Mutex<Vec<Arc<dyn Service + Send + Sync + 'static>>>,
    is_running: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct Runtime {
    inner: Arc<Inner>,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime::new(&[])
    }
}

impl Runtime {
    pub fn new(services: &[Arc<dyn Service + Send + Sync + 'static>]) -> Self {
        let runtime = Self {
            inner: Arc::new(Inner {
                services: Mutex::new(services.to_vec()),
                is_running: Arc::new(AtomicBool::new(false)),
            }),
        };

        register_global(Some(runtime.clone()));

        runtime
    }

    pub fn register_service(&self, service: Arc<dyn Service + Send + Sync + 'static>) {
        self.inner.services.lock().unwrap().push(service);
    }

    pub fn start(&self) {
        self.inner.is_running.store(true, Ordering::SeqCst);
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
        self.inner.is_running.store(false, Ordering::SeqCst);
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
