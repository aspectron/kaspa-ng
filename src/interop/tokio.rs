use crate::imports::*;
// use std::thread::JoinHandle;
use crate::interop::AsyncService;
use crate::result::Result;
use futures_util::future::{select_all, try_join_all};

pub struct AsyncRuntime {
    threads : usize,
    services: Mutex<Vec<Arc<dyn AsyncService>>>,
    worker: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl Default for AsyncRuntime {
    fn default() -> Self {
        // TODO
        Self::new(std::cmp::max(num_cpus::get() / 3, 2))
    }
}

impl AsyncRuntime {

    pub fn new(threads: usize) -> Self {
        // trace!("Creating the async-runtime service");
        Self { threads, services: Mutex::new(Vec::new()), worker: Mutex::new(None) }
    }

    pub fn register<T>(&self, service: Arc<T>)
    where
        T: AsyncService + 'static,
    {
        // trace!("async-runtime registering service {}", service.clone().ident());
        self.services.lock().unwrap().push(service);
    }

    pub fn spawn(self: &Arc<AsyncRuntime>) {
        // trace!("initializing async-runtime service");
        let this = self.clone();
        self.worker.lock().unwrap().replace(std::thread::Builder::new().name("interop".to_string()).spawn(move || this.worker()).unwrap());
    }

    pub fn join(self: &Arc<AsyncRuntime>) {
        // self.shutdown();
        if let Some(worker) = self.worker.lock().unwrap().take() {
            match worker.join() {
                Ok(()) => {}
                Err(err) => {
                    println!("thread join failure: {:?}", err);
                }
            }
        }

        // trace!("... core is shut down");
    }

    /// Launch a tokio Runtime and run the top-level async objects

    // pub fn spawn(self: Arc<AsyncRuntime>) -> JoinHandle<()> {
    //     std::thread::spawn(move || self.worker())
    //     //    // trace!("spawning async-runtime service");
    //     // std::thread::Builder::new().name("interop".to_string()).spawn(move || self.worker()).unwrap()
    // }
    
    pub fn worker(self: &Arc<AsyncRuntime>) {
        return tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.threads)
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(async { self.worker_impl().await });
    }

    pub async fn worker_impl(self: &Arc<AsyncRuntime>) {
        let rt_handle = tokio::runtime::Handle::current();
        std::thread::spawn(move || loop {
            // See https://github.com/tokio-rs/tokio/issues/4730 and comment therein referring to
            // https://gist.github.com/Darksonn/330f2aa771f95b5008ddd4864f5eb9e9#file-main-rs-L6
            // In our case it's hard to avoid some short blocking i/o calls to the DB so we place this
            // workaround for now to avoid any rare yet possible system freeze.
            std::thread::sleep(std::time::Duration::from_secs(2));
            rt_handle.spawn(std::future::ready(()));
        });

        // Start all async services
        // All services futures are spawned as tokio tasks to enable parallelism
        // trace!("async-runtime worker starting");
        let futures = self
            .services
            .lock()
            .unwrap()
            .iter()
            .map(|x| tokio::spawn(x.clone().start()))
            .collect::<Vec<tokio::task::JoinHandle<Result<()>>>>();

        // wait for at least one service to return
        let (result, idx, remaining_futures) = select_all(futures).await;
        // trace!("async-runtime worker had service {} returning", self.services.lock().unwrap()[idx].clone().ident());
        // if at least one service yields an error, initiate global shutdown
        // this will cause signal_exit() to be executed externally (by Core invoking `stop()`)
        match result {
            Ok(Err(_)) | Err(_) => {
                // trace!("shutting down core due to async-runtime error");
                // core.shutdown()
            }
            _ => {}
        }

        // wait for remaining services to finish
        // trace!("async-runtime worker joining remaining {} services", remaining_futures.len());
        try_join_all(remaining_futures).await.unwrap();

        // Stop all async services
        // All services futures are spawned as tokio tasks to enable parallelism
        let futures = self
            .services
            .lock()
            .unwrap()
            .iter()
            .map(|x| tokio::spawn(x.clone().stop()))
            .collect::<Vec<tokio::task::JoinHandle<Result<()>>>>();
        try_join_all(futures).await.unwrap();

        // trace!("async-runtime worker stopped");
    }

    pub fn shutdown(self: &Arc<AsyncRuntime>) {
        // trace!("Sending an exit signal to all async-runtime services");
        for service in self.services.lock().unwrap().iter() {
            service.clone().signal_exit();
        }
    }
}