use crate::imports::*;
use crate::runtime::services::kaspa::Config;
use kaspa_core::core::Core;
use kaspa_core::signals::Shutdown;
use kaspa_utils::fd_budget;
use kaspa_wallet_core::rpc::DynRpcApi;
use kaspad_lib::args::Args;
use kaspad_lib::daemon::{create_core_with_runtime, Runtime as KaspadRuntime};

struct Inner {
    thread: std::thread::JoinHandle<()>,
    core: Arc<Core>,
    rpc_core_service: Option<Arc<DynRpcApi>>,
}

#[derive(Default)]
pub struct InProc {
    inner: Arc<Mutex<Option<Inner>>>,
}

impl InProc {
    pub fn rpc_core_services(&self) -> Option<Arc<DynRpcApi>> {
        if let Some(inner) = self.inner.lock().unwrap().as_ref() {
            inner.rpc_core_service.clone()
        } else {
            None
        }
    }
}

#[async_trait]
impl super::Kaspad for InProc {
    async fn start(self: Arc<Self>, config: Config) -> Result<()> {
        let args = Args::try_from(config)?;

        let fd_total_budget = fd_budget::limit()
            - args.rpc_max_clients as i32
            - args.inbound_limit as i32
            - args.outbound_target as i32;

        let runtime = KaspadRuntime::default();
        let (core, rpc_core_service) = create_core_with_runtime(&runtime, &args, fd_total_budget);
        let core_ = core.clone();
        let thread = std::thread::Builder::new()
            .name("kaspad".to_string())
            .spawn(move || {
                core_.run();
            })?;
        self.inner.lock().unwrap().replace(Inner {
            thread,
            core,
            rpc_core_service: Some(rpc_core_service),
        });
        Ok(())
    }

    async fn stop(self: Arc<Self>) -> Result<()> {
        if let Some(mut inner) = self.inner.lock().unwrap().take() {
            let (core, thread) = {
                let rpc_core_service = inner.rpc_core_service.take();
                drop(rpc_core_service);
                (inner.core, inner.thread)
            };
            core.shutdown();
            drop(core);
            thread
                .join()
                .map_err(|_| Error::custom("kaspad inproc thread join failure"))?;
        }
        Ok(())
    }
}
