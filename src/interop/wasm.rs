#![allow(dead_code)]

use crate::interop::*;
use futures_util::future::join_all;
use workflow_core::task::spawn;

pub struct AsyncRuntime {
    services: Mutex<Vec<Arc<dyn AsyncService>>>,
    handles: Mutex<Vec<Receiver<()>>>,
}

impl Default for AsyncRuntime {
    fn default() -> Self {
        AsyncRuntime {
            services : Mutex::new(Vec::new()),
            handles : Mutex::new(Vec::new())
        }
    }
}

impl AsyncRuntime {


    pub fn register<T>(&self, service: Arc<T>)
    where
        T: AsyncService + 'static,
    {
        self.services.lock().unwrap().push(service);
    }

    pub fn spawn(self: &Arc<AsyncRuntime>) {
        self.services.lock().unwrap().iter().for_each(|service| {
            let (tx, rx) = oneshot();
            self.handles.lock().unwrap().push(rx);
            let service = service.clone();
            spawn(async move {
                service.start().await.expect("service start error");
                tx.send(()).await.unwrap();
            })
        });
    }
    
    pub fn shutdown(self: &Arc<AsyncRuntime>) {
        for service in self.services.lock().unwrap().iter() {
            service.clone().signal_exit();
        }
    }

    pub async fn join(self: &Arc<AsyncRuntime>) {
        let handles = self.handles.lock().unwrap().clone();
        let handles = handles.iter().map(|rx|rx.recv()).collect::<Vec<_>>();
        join_all(handles).await;
    }
}
