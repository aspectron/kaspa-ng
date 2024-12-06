use crate::imports::*;

pub mod repaint_service;
pub use repaint_service::RepaintService;

pub mod kaspa;
pub use kaspa::KaspaService;

pub mod peer_monitor;
pub use peer_monitor::PeerMonitorService;

pub mod feerate_monitor;
pub use feerate_monitor::FeerateMonitorService;

pub mod market_monitor;
pub use market_monitor::MarketMonitorService;

pub mod update_monitor;
pub use update_monitor::UpdateMonitorService;

pub mod metrics_monitor;
pub use metrics_monitor::MetricsService;
cfg_if! {
    if #[cfg(not(feature = "lean"))] {

        pub mod blockdag_monitor;
        pub use blockdag_monitor::BlockDagMonitorService;
    }
}

/// Service is a core component of the Kaspa NG application responsible for
/// running application services and communication between these services.
#[async_trait]
pub trait Service: Sync + Send {
    fn name(&self) -> &'static str;

    /// Start the service
    async fn spawn(self: Arc<Self>) -> Result<()>;

    /// Signal the service termination (post a shutdown request)
    fn terminate(self: Arc<Self>);

    /// Block until the service is terminated
    async fn join(self: Arc<Self>) -> Result<()>;

    /// Called when Kaspa RPC API has been created (but node is not
    /// connected yet, see [`connect_rpc`](Service::connect_rpc))
    /// for connectivity signalling.
    async fn attach_rpc(self: Arc<Self>, _rpc_api: &Arc<dyn RpcApi>) -> Result<()> {
        Ok(())
    }
    /// Called when Kaspa RPC API is no longer available
    async fn detach_rpc(self: Arc<Self>) -> Result<()> {
        Ok(())
    }

    /// Called when Kaspa RPC API has successfully connected
    async fn connect_rpc(self: Arc<Self>) -> Result<()> {
        Ok(())
    }

    /// Called when Kaspa RPC API has disconnected
    async fn disconnect_rpc(self: Arc<Self>) -> Result<()> {
        Ok(())
    }
}
