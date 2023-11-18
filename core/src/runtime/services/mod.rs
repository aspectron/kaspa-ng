use crate::imports::*;

pub mod kaspa;
pub use kaspa::KaspaService;

pub mod peer_monitor;
pub use peer_monitor::PeerMonitorService;

pub mod metrics_monitor;
pub use metrics_monitor::MetricsService;

pub mod plugin_manager;
pub use plugin_manager::PluginManagerService;

/// Service is a core component of the Kaspa NG application responsible for
/// running application services and communication between these services.
#[async_trait]
pub trait Service: Sync + Send {
    /// Start the service
    async fn spawn(self: Arc<Self>) -> Result<()>;

    /// Signal the service termination (post a shutdown request)
    fn terminate(self: Arc<Self>);

    /// Block until the service is terminated
    async fn join(self: Arc<Self>) -> Result<()>;

    /// Called when Kaspa RPC API is available (connection to the node is established successfully)
    async fn attach_rpc(self: Arc<Self>, _rpc_api: &Arc<dyn RpcApi>) -> Result<()> {
        Ok(())
    }
    /// Called when Kaspa RPC API is no longer available (node is disconnected)
    async fn detach_rpc(self: Arc<Self>) -> Result<()> {
        Ok(())
    }
}
