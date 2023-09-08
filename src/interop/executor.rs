use crate::imports::*;
pub use futures::{future::FutureExt, select, Future};
use std::{
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::interop::AsyncService;

// pub type TaskResult<T> = std::result::Result<T, Error>;

// // pub type TaskFn<A, T> = Arc<Box<dyn Send + Sync + Fn(A, Receiver<()>) -> FnReturn<T> + 'static>>;
// pub type FnReturn<T> = Pin<Box<(dyn Send + Sync + 'static + Future<Output = T>)>>;

// pub fn spawn<FN>(task_fn: FN)
// where
//     FN: Send + Sync + Fn(Args) -> FnReturn<T> + 'static,
// {
//     Self::new_with_boxed_task_fn(Box::new(task_fn))
// }

static mut SENDER: Option<Sender<ExecutorEvents>> = None;

// type NonblockingFuture<T> = Pin<Box<dyn Future<Output = T> + Send + Sync>>;
type NonblockingFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

// pub fn spawn<F, T>(future: F)
pub fn spawn<F>(future: F)
where
    // F: Future<Output = T> + Send + 'static,
    // F: Future<Output = ()> + Send + 'static,
    F: Future<Output = Result<()>> + Send + 'static,
    // T: Send + 'static,
    // T: Send + 'static,
{
    unsafe {
        if let Some(sender) = &SENDER {
            sender
                .try_send(ExecutorEvents::Spawn(Box::pin(future)))
                .unwrap();
        } else {
            panic!("Unable to spawn non-blocking future - executor service is not initialized")
        }
    }
    // tokio::task::spawn(future);
}

// fn new_with_boxed_task_fn<FN>(task_fn: Box<FN>) -> Task<A, T>
// where
//     FN: Send + Sync + Fn(A, Receiver<()>) -> FnReturn<T> + 'static,
// {
//     Task {
//         inner: Arc::new(TaskInner::new_with_boxed_task_fn(task_fn)),
//     }
// }

// #[derive(Debug)]
pub enum ExecutorEvents {
    Spawn(NonblockingFuture<Result<()>>),
    // Open { name : Option<String>, secret : Secret },
    Exit,
}

pub struct Executor {
    pub application_events: interop::Channel<Events>,
    pub executor_events: Channel<ExecutorEvents>,
    pub shutdown: AtomicBool,
    // pub wallet : Arc<runtime::Wallet>,
}

impl Executor {
    pub fn new(application_events: interop::Channel<crate::events::Events>) -> Self {
        let executor_events = Channel::unbounded();

        unsafe {
            SENDER = Some(executor_events.sender.clone());
        }

        Self {
            application_events,
            executor_events,
            shutdown: AtomicBool::new(false),
        }
    }

    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        unsafe {
            SENDER = None;
        }
    }
}

impl AsyncService for Executor {
    fn start(self: Arc<Self>) -> BoxFuture<'static, Result<()>> {
        println!("executor relay starting...");
        let this = self.clone();
        let application_events_sender = self.application_events.sender.clone();
        Box::pin(async move {
            // println!("starting wallet...");
            // this.wallet.start().await.unwrap_or_else(|err| {
            //     println!("Wallet start error: {:?}", err);
            // });

            loop {
                select! {
                    msg = this.as_ref().executor_events.receiver.recv().fuse() => {
                        // println!("Wallet received message: {:?}", msg);

                        if let Ok(event) = msg {
                            match event {
                                ExecutorEvents::Spawn(task) => {
                                    let sender = application_events_sender.clone();
                                    workflow_core::task::spawn(async move {
                                        if let Err(err) = task.await {
                                            sender.send(Events::Error(Box::new(err.to_string()))).await.unwrap();
                                            println!("spawned task error: {:?}", err);
                                        }
                                    });
                                },
                                ExecutorEvents::Exit => {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }
            }

            Ok(())
        })
    }

    fn signal_exit(self: Arc<Self>) {
        self.executor_events
            .sender
            .try_send(ExecutorEvents::Exit)
            .unwrap();
    }

    fn stop(self: Arc<Self>) -> BoxFuture<'static, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
}
