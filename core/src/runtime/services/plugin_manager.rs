use crate::imports::*;
use crate::runtime::plugins::*;

pub enum PluginManagerEvents {
    Start(Arc<dyn Plugin>),
    Stop(Arc<dyn Plugin>),
    Exit,
}

pub struct PluginManagerService {
    pub application_events: ApplicationEventsChannel,
    pub service_events: Channel<PluginManagerEvents>,
    pub task_ctl: Channel<()>,
    pub plugins: Arc<Vec<Arc<dyn Plugin>>>,
    pub plugin_settings: Mutex<PluginSettingsMap>,
    pub running_plugins: HashMap<String, AtomicBool>,
}

impl PluginManagerService {
    pub fn new(application_events: ApplicationEventsChannel, settings: &Settings) -> Self {
        let plugins: Vec<Arc<dyn Plugin>> = vec![Arc::new(MarketMonitorPlugin::new(
            application_events.clone(),
        ))];

        let running_plugins = plugins
            .iter()
            .map(|plugin| (plugin.ident().to_string(), AtomicBool::new(false)))
            .collect::<HashMap<_, _>>();

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
            running_plugins,
            // is_enabled_map: HashMap::new(),
        }
    }

    pub fn is_running(&self, plugin: &Arc<dyn Plugin>) -> bool {
        self.running_plugins
            .get(plugin.ident())
            .map(|is_running| is_running.load(Ordering::Relaxed))
            .expect("is_running(): no such plugin")
    }

    pub fn is_enabled(&self, plugin: &Arc<dyn Plugin>) -> bool {
        self.plugin_settings
            .lock()
            .unwrap()
            .get(plugin.ident())
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    pub fn enable(&self, plugin: &Arc<dyn Plugin>, enabled: bool) {
        if let Some(plugin_settings) = self.plugin_settings.lock().unwrap().get_mut(plugin.ident())
        {
            plugin_settings.enabled = enabled;
        }

        // - TODO - START PLUGIN IF NOT RUNNING
        // - TODO - START PLUGIN IF NOT RUNNING
        // - TODO - START PLUGIN IF NOT RUNNING
        // - TODO - START PLUGIN IF NOT RUNNING
        // - TODO - START PLUGIN IF NOT RUNNING
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
    fn name(&self) -> &'static str {
        "plugin-manager"
    }

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
        let this = self.clone();
        let _application_events_sender = self.application_events.sender.clone();

        loop {
            select! {

                msg = this.service_events.recv().fuse() => {
                    if let Ok(event) = msg {
                        match event {
                            PluginManagerEvents::Start(_plugin) => {
                            }
                            PluginManagerEvents::Stop(_plugin) => {
                            }
                            PluginManagerEvents::Exit => {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        this.task_ctl.send(()).await.unwrap();

        Ok(())
    }

    fn terminate(self: Arc<Self>) {
        self.terminate_plugins();

        self.service_events
            .sender
            .try_send(PluginManagerEvents::Exit)
            .unwrap();
    }

    async fn join(self: Arc<Self>) -> Result<()> {
        self.join_plugins().await?;
        Ok(())
    }
}
