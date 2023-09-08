use crate::imports::*;
use kaspa_core::core::Core;
use kaspa_core::signals::Shutdown;
use kaspa_rpc_service::service::RpcCoreService;
use kaspa_wallet_core::DynRpcApi;
use kaspad::args::Args;
use kaspad::daemon::create_core;

struct Inner {
    thread: std::thread::JoinHandle<()>,
    core: Arc<Core>,
    rpc_core_service: Arc<RpcCoreService>,
}

#[derive(Default)]
pub struct InProc {
    inner: Arc<Mutex<Option<Inner>>>,
}

impl InProc {
    pub fn rpc_core_services(&self) -> Option<Arc<DynRpcApi>> {
        if let Some(inner) = self.inner.lock().unwrap().as_ref() {
            Some(inner.rpc_core_service.clone())
        } else {
            None
        }
    }
}

impl super::Kaspad for InProc {
    fn start(&self, args: Args) -> Result<()> {
        let (core, rpc_core_service) = create_core(args);
        // self.inner.core.lock().unwrap().replace(core.clone());
        let core_ = core.clone();
        let thread = std::thread::Builder::new()
            .name("kaspad".to_string())
            .spawn(move || {
                core_.run();
            })?;
        // self.inner.thread.lock().unwrap().replace(thread);
        self.inner.lock().unwrap().replace(Inner {
            thread,
            core,
            rpc_core_service,
        });
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        if let Some(inner) = self.inner.lock().unwrap().take() {
            inner.core.shutdown();
            inner
                .thread
                .join()
                .map_err(|_| Error::custom("kaspad inproc thread join failure"))?;
        }
        Ok(())
    }
}
