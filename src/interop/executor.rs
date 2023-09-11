use crate::imports::*;
pub use futures::{future::FutureExt, select, Future};
use std::{
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::interop::AsyncService;

static mut SENDER: Option<Sender<ExecutorEvents>> = None;

type NonblockingFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
pub fn spawn<F>(future: F)
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    let sender = application_events_sender.clone();
    workflow_core::task::spawn(async move {
        if let Err(err) = task.await {
            sender.send(Events::Error(Box::new(err.to_string()))).await.unwrap();
            println!("spawned task error: {:?}", err);
        }
    });

    // unsafe {
    //     if let Some(sender) = &SENDER {
    //         sender
    //             .try_send(ExecutorEvents::Spawn(Box::pin(future)))
    //             .unwrap();
    //     } else {
    //         panic!("Unable to spawn non-blocking future - executor service is not initialized")
    //     }
    // }
}

pub enum ExecutorEvents {
    Spawn(NonblockingFuture<Result<()>>),
    Exit,
}

pub struct Executor {
    pub application_events: interop::Channel<Events>,
    pub executor_events: Channel<ExecutorEvents>,
    pub shutdown: AtomicBool,
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
    fn start_service(self: Arc<Self>) -> BoxFuture<'static, Result<()>> {
        println!("executor relay starting...");
        let this = self.clone();
        let application_events_sender = self.application_events.sender.clone();
        Box::pin(async move {
            loop {
                select! {
                    msg = this.as_ref().executor_events.receiver.recv().fuse() => {

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

    fn signal_termination(self: Arc<Self>) {
        self.executor_events
            .sender
            .try_send(ExecutorEvents::Exit)
            .unwrap();
    }

    fn stop_service(self: Arc<Self>) -> BoxFuture<'static, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
}
