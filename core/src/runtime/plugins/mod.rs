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

// / PluginContainer is a wrapper around a plugin instance that
// / tracks the plugin running state as plugins can be started
// / and stopped during the application runtime.
// pub struct PluginContainer {
//     pub plugin: Arc<dyn Plugin>,
//     pub is_running: Arc<AtomicBool>,
// }

// impl PluginContainer {
//     pub fn new(plugin: Arc<dyn Plugin>) -> Self {
//         Self {
//             plugin,
//             is_running: Arc::new(AtomicBool::new(false)),
//         }
//     }

//     pub fn ident(&self) -> &'static str {
//         self.plugin.ident()
//     }

//     pub fn name(&self) -> &'static str {
//         self.plugin.name()
//     }

//     pub fn render(&self, ui: &mut Ui) {
//         self.plugin.render(ui);
//     }

//     pub fn store(&self) -> Result<Option<serde_json::Value>> {
//         self.plugin.store()
//     }

//     pub fn load(&self, settings: serde_json::Value) -> Result<()> {
//         self.plugin.load(settings)
//     }

//     pub fn is_running(&self) -> bool {
//         self.is_running.load(Ordering::SeqCst)
//     }

//     pub fn is_enabled(&self) -> bool {
//         self.plugin.is_enabled()
//     }

//     pub async fn start(&self) -> Result<()> {
//         if self.plugin.is_enabled() && !self.is_running.load(Ordering::SeqCst) {
//             self.is_running.store(true, Ordering::SeqCst);
//             self.plugin.clone().start().await?;
//         }
//         Ok(())
//     }

//     pub fn terminate(&self) {
//         if self.is_running.load(Ordering::SeqCst) {
//             self.plugin.clone().terminate();
//         }
//     }

//     pub async fn join(&self) -> Result<()> {
//         if self.is_running.load(Ordering::SeqCst) {
//             self.is_running.store(false, Ordering::SeqCst);
//             self.plugin.clone().join().await?;
//         }
//         Ok(())
//     }

//     pub async fn attach_rpc(&self, rpc_api: &Arc<dyn RpcApi>) -> Result<()> {
//         self.plugin.clone().attach_rpc(rpc_api).await
//     }

//     pub async fn detach_rpc(&self) -> Result<()> {
//         self.plugin.clone().detach_rpc().await
//     }
// }

// impl From<Arc<dyn Plugin>> for PluginContainer {
//     fn from(plugin: Arc<dyn Plugin>) -> Self {
//         Self::new(plugin)
//     }
// }
