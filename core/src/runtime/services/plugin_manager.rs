use crate::imports::*;
// pub use futures::{future::FutureExt, select, Future};
// use kaspa_rpc_core::RpcPeerInfo;

// pub const PEER_POLLING_INTERVAL: usize = 1; // 1 sec

// use crate::runtime::plugins::Plugin;
use crate::runtime::plugins::*;

pub enum PluginManagerEvents {
    Exit,
}

pub struct PluginManagerService {
    pub application_events: ApplicationEventsChannel,
    pub service_events: Channel<PluginManagerEvents>,
    pub task_ctl: Channel<()>,
    // pub rpc_api: Mutex<Option<Arc<dyn RpcApi>>>,
    // pub peer_info: Mutex<Option<Arc<Vec<RpcPeerInfo>>>>,
    pub plugins: Arc<Vec<Arc<dyn Plugin>>>,
    // pub plugins_running : HashMap<String,AtomicBool>,
    pub plugin_settings: Mutex<PluginSettingsMap>,
    pub is_running_map: HashMap<String, AtomicBool>,
    // pub is_enabled_map: HashMap<String, AtomicBool>,
}

impl PluginManagerService {
    pub fn new(application_events: ApplicationEventsChannel, settings: &Settings) -> Self {
        let plugins: Vec<Arc<dyn Plugin>> = vec![Arc::new(MarketMonitorPlugin::new(
            application_events.clone(),
        ))];

        // let plugins: Vec<PluginContainer> = plugins
        //     .into_iter()
        //     .map(|plugin| PluginContainer::new(plugin))
        //     .collect();

        Self {
            application_events,
            service_events: Channel::unbounded(),
            task_ctl: Channel::oneshot(),
            plugins: Arc::new(plugins),
            plugin_settings: Mutex::new(settings.plugins.clone().unwrap_or_default()),
            is_running_map: HashMap::new(),
            // is_enabled_map: HashMap::new(),
        }
    }

    pub fn is_running(&self, plugin: &Arc<dyn Plugin>) -> bool {
        self.is_running_map
            .get(plugin.ident())
            .map(|is_running| is_running.load(Ordering::Relaxed))
            .unwrap_or(false)
    }

    pub fn is_enabled(&self, plugin: &Arc<dyn Plugin>) -> bool {
        self.plugin_settings
            .lock()
            .unwrap()
            .get(plugin.ident())
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    pub fn plugins(&self) -> &Arc<Vec<Arc<dyn Plugin>>> {
        &self.plugins
    }

    pub fn running_plugins(&self) -> Vec<Arc<dyn Plugin>> {
        self.plugins()
            .iter()
            .filter(|plugin| self.is_running(plugin))
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn store(&self) -> Result<PluginSettingsMap> {
        let mut plugin_settings_map = PluginSettingsMap::default();
        for plugin in self.plugins().iter() {
            let enabled = self.is_enabled(plugin);
            if let Ok(Some(settings)) = plugin.store() {
                let plugin_settings = PluginSettings { enabled, settings };
                plugin_settings_map.insert(plugin.ident().to_string(), plugin_settings);
            }
        }
        Ok(plugin_settings_map)
    }

    pub async fn start_plugins(&self, settings: &Settings) -> Result<()> {
        // do nothing if plugins are disabled
        if !settings.enable_plugins {
            return Ok(());
        }

        *self.plugin_settings.lock().unwrap() = settings.plugins.clone().unwrap_or_default();

        let running_plugins = self.running_plugins();
        for plugin in running_plugins.into_iter() {
            let ident = plugin.ident();
            if let Some(plugin_settings) = self.plugin_settings.lock().unwrap().get(ident).cloned()
            {
                plugin.load(plugin_settings.settings)?;
            }
            if let Err(err) = plugin.clone().start().await {
                log_error!("Failed to start plugin {}: {}", plugin.name(), err);
            }
        }
        Ok(())
    }

    pub fn terminate_plugins(&self) {
        let running_plugins = self.running_plugins();
        running_plugins
            .into_iter()
            .for_each(|plugin| plugin.terminate());
    }

    pub async fn join_plugins(&self) -> Result<()> {
        let running_plugins = self.running_plugins();
        for plugin in running_plugins.into_iter() {
            if let Err(err) = plugin.clone().join().await {
                log_error!("Failed to join plugin {}: {}", plugin.name(), err);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Service for PluginManagerService {
    async fn attach_rpc(self: Arc<Self>, rpc_api: &Arc<dyn RpcApi>) -> Result<()> {
        let running_plugins = self.running_plugins();
        for plugin in running_plugins.into_iter() {
            if let Err(err) = plugin.clone().attach_rpc(rpc_api).await {
                log_error!("Failed to attach RPC to plugin {}: {}", plugin.name(), err);
            }
        }
        Ok(())
    }

    async fn detach_rpc(self: Arc<Self>) -> Result<()> {
        let running_plugins = self.running_plugins();
        for plugin in running_plugins.into_iter() {
            if let Err(err) = plugin.clone().detach_rpc().await {
                log_error!(
                    "Failed to detach RPC from plugin {}: {}",
                    plugin.name(),
                    err
                );
            }
        }

        Ok(())
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        Ok(())
    }

    fn terminate(self: Arc<Self>) {
        self.terminate_plugins();
    }

    async fn join(self: Arc<Self>) -> Result<()> {
        self.join_plugins().await?;
        Ok(())
    }
}
