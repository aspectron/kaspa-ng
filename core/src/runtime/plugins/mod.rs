use crate::imports::*;

pub mod market_monitor;
pub use market_monitor::MarketMonitorPlugin;

#[async_trait]
pub trait Plugin: Sync + Send {
    /// Short identifier of the plugin (used for storage of options in the application settings)
    fn ident(&self) -> &'static str;

    /// Human-readable name of the plugin
    fn name(&self) -> &'static str;

    /// User interface rendering of the plugin within the settings panel
    fn render(&self, ui: &mut Ui);

    fn store(&self) -> Result<Option<serde_json::Value>>;
    fn load(&self, data: serde_json::Value) -> Result<()>;

    /// Indicates if a plugin is currently enabled. The plugin
    /// will not be started if it is not enabled.
    // fn is_enabled(&self) -> bool;

    /// Called when the plugin needs to be started
    async fn start(self: Arc<Self>) -> Result<()>;

    /// Signal the plugin termination (post a shutdown request)
    fn terminate(self: Arc<Self>);

    /// Block until the plugin is terminated
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
