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

pub mod payload;
pub use payload::Payload;

pub struct Inner {
    application_events: ApplicationEventsChannel,
    kaspa: Arc<KaspaService>,
    runtime: Runtime,
    egui_ctx: egui::Context,
    is_running: Arc<AtomicBool>,
}

/// Interop is a core component of the Kaspa NG application responsible for
/// running application services and communication between these services
/// and the application UI.
#[derive(Clone)]
pub struct Interop {
    inner: Arc<Inner>,
}

impl Interop {
    pub fn new(egui_ctx: &egui::Context, settings: &Settings) -> Self {
        let application_events = ApplicationEventsChannel::unbounded(Some(egui_ctx.clone()));
        let kaspa = Arc::new(KaspaService::new(application_events.clone(), settings));
        let runtime = Runtime::new(&[kaspa.clone()]);

        let interop = Self {
            inner: Arc::new(Inner {
                application_events,
                kaspa,
                runtime,
                egui_ctx: egui_ctx.clone(),
                is_running: Arc::new(AtomicBool::new(false)),
            }),
        };

        register_global(Some(interop.clone()));

        interop
    }

    /// Get a reference to the interop runtime.
    pub fn runtime(&self) -> &Runtime {
        &self.inner.runtime
    }

    /// Start the interop runtime.
    pub fn start(&self) {
        self.inner.is_running.store(true, Ordering::SeqCst);
        self.runtime().start();
    }

    /// Shutdown interop runtime.
    pub async fn shutdown(&self) {
        if self.inner.is_running.load(Ordering::SeqCst) {
            self.inner.is_running.store(false, Ordering::SeqCst);
            self.runtime().shutdown();
            self.runtime().join().await;
            register_global(None);
        }
    }

    /// Returns the reference to the wallet API.
    pub fn wallet(&self) -> Arc<dyn WalletApi> {
        self.inner.kaspa.wallet()
    }

    /// Returns the reference to the kaspa service.
    pub fn kaspa_service(&self) -> &Arc<KaspaService> {
        &self.inner.kaspa
    }

    /// Returns the reference to the application events channel.
    pub fn application_events(&self) -> &ApplicationEventsChannel {
        &self.inner.application_events
    }

    /// Send an application even to the UI asynchronously.
    pub async fn send(&self, msg: Events) -> Result<()> {
        self.inner.application_events.sender.send(msg).await?;
        Ok(())
    }

    /// Send an application event to the UI synchronously.
    pub fn try_send(&self, msg: Events) -> Result<()> {
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

    pub fn egui_ctx(&self) -> &egui::Context {
        &self.inner.egui_ctx
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

/// Spawn an async task that will result in 
/// egui redraw upon its completion.
pub fn spawn<F>(task: F)
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    interop().spawn_task(task);
}

/// Spawn an async task that will result in 
/// egui redraw upon its completion. Upon
/// the task completion, the supplied [`Payload`]
/// will be populated with the task result.
pub fn spawn_with_result<R, F>(payload: &Payload<std::result::Result<R, Error>>, task: F)
where
    R: Clone + Send + 'static,
    F: Future<Output = Result<R>> + Send + 'static,
{
    interop().spawn_task_with_result(payload, task);
}

/// Gracefully halt the interop runtime. This is used
/// to shutdown kaspad when the kaspa-ng process exit
/// is an inevitable eventuality.
pub fn halt() {
    let handle = tokio::spawn(async move { interop().shutdown().await });

    while !handle.is_finished() {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

/// Attempt to halt the interop runtime but exit the process 
/// if it takes too long. This is used in attempt to shutdown 
/// kaspad if the kaspa-ng process panics, which can result
/// in a still functioning zombie child process on unix systems.
pub fn abort() {

    const TIMEOUT: u128 = 5000;
    let flag = Arc::new(AtomicBool::new(false));
    let flag_ = flag.clone();
    let thread = std::thread::Builder::new()
        .name("halt".to_string())
        .spawn(move || {
            let start = std::time::Instant::now();
            while !flag_.load(Ordering::SeqCst) {
                if start.elapsed().as_millis() > TIMEOUT {
                    println!("halting...");
                    std::process::exit(1);
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }).ok();

    halt();

    flag.store(true, Ordering::SeqCst);
    if let Some(thread) = thread {
        thread.join().unwrap();
    }

    std::process::exit(1);

}