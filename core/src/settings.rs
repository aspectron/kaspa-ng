use crate::imports::*;
use kaspa_metrics_core::Metric;
use kaspa_utils::networking::ContextualNetAddress;
use kaspa_wallet_core::storage::local::storage::Storage;
use kaspa_wrpc_client::WrpcEncoding;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        #[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
        #[serde(rename_all = "kebab-case")]
        pub enum KaspadNodeKind {
            Disable,
            Remote,
            IntegratedInProc,
            #[default]
            IntegratedAsDaemon,
            ExternalAsDaemon,
        }

        const KASPAD_NODE_KINDS: [KaspadNodeKind; 5] = [
            KaspadNodeKind::Disable,
            KaspadNodeKind::Remote,
            KaspadNodeKind::IntegratedInProc,
            KaspadNodeKind::IntegratedAsDaemon,
            KaspadNodeKind::ExternalAsDaemon,
        ];

        impl std::fmt::Display for KaspadNodeKind {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    KaspadNodeKind::Disable => write!(f, "Disabled"),
                    KaspadNodeKind::Remote => write!(f, "Remote"),
                    KaspadNodeKind::IntegratedInProc => write!(f, "Integrated Node"),
                    KaspadNodeKind::IntegratedAsDaemon => write!(f, "Integrated Daemon"),
                    KaspadNodeKind::ExternalAsDaemon => write!(f, "External Daemon"),
                }
            }
        }

    } else {
        #[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
        #[serde(rename_all = "kebab-case")]
        pub enum KaspadNodeKind {
            Disable,
            #[default]
            Remote,
        }

        const KASPAD_NODE_KINDS: [KaspadNodeKind; 1] = [
            KaspadNodeKind::Remote,
        ];

        impl std::fmt::Display for KaspadNodeKind {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    KaspadNodeKind::Disable => write!(f, "Disable"),
                    KaspadNodeKind::Remote => write!(f, "Remote"),
                }
            }
        }
    }
}

impl KaspadNodeKind {
    pub fn iter() -> impl Iterator<Item = &'static KaspadNodeKind> {
        KASPAD_NODE_KINDS.iter()
    }

    pub fn describe(&self) -> &str {
        match self {
            KaspadNodeKind::Disable => i18n("Disables node connectivity (Offline Mode)."),
            KaspadNodeKind::Remote => i18n("Connects to a Remote Rusty Kaspa Node via wRPC."),
            #[cfg(not(target_arch = "wasm32"))]
            KaspadNodeKind::IntegratedInProc => i18n("The node runs as a part of the Kaspa-NG application process. This reduces communication overhead (experimental)."),
            #[cfg(not(target_arch = "wasm32"))]
            KaspadNodeKind::IntegratedAsDaemon => i18n("The node is spawned as a child daemon process (recommended)."),
            #[cfg(not(target_arch = "wasm32"))]
            KaspadNodeKind::ExternalAsDaemon => i18n("A binary at another location is spawned a child process (experimental, for development purposes only)."),
        }
    }

    pub fn is_config_capable(&self) -> bool {
        match self {
            KaspadNodeKind::Disable => false,
            KaspadNodeKind::Remote => false,
            #[cfg(not(target_arch = "wasm32"))]
            KaspadNodeKind::IntegratedInProc => true,
            #[cfg(not(target_arch = "wasm32"))]
            KaspadNodeKind::IntegratedAsDaemon => true,
            #[cfg(not(target_arch = "wasm32"))]
            KaspadNodeKind::ExternalAsDaemon => true,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum RpcKind {
    #[default]
    Wrpc,
    Grpc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RpcConfig {
    Wrpc {
        url: Option<String>,
        encoding: WrpcEncoding,
    },
    Grpc {
        url: Option<NetworkInterfaceConfig>,
    },
}

impl Default for RpcConfig {
    fn default() -> Self {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                let url = "127.0.0.1";
            } else {
                use workflow_dom::utils::*;
                let url = window().location().hostname().expect("KaspadNodeKind: Unable to get hostname");
            }
        }
        RpcConfig::Wrpc {
            url: Some(url.to_string()),
            encoding: WrpcEncoding::Borsh,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkInterfaceKind {
    #[default]
    Local,
    Any,
    Custom,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NetworkInterfaceConfig {
    #[serde(rename = "type")]
    pub kind: NetworkInterfaceKind,
    pub custom: ContextualNetAddress,
}

impl Default for NetworkInterfaceConfig {
    fn default() -> Self {
        Self {
            kind: NetworkInterfaceKind::Local,
            custom: ContextualNetAddress::loopback(),
        }
    }
}

impl From<NetworkInterfaceConfig> for ContextualNetAddress {
    fn from(network_interface_config: NetworkInterfaceConfig) -> Self {
        match network_interface_config.kind {
            NetworkInterfaceKind::Local => "127.0.0.1".parse().unwrap(),
            NetworkInterfaceKind::Any => "0.0.0.0".parse().unwrap(),
            NetworkInterfaceKind::Custom => network_interface_config.custom,
        }
    }
}

impl std::fmt::Display for NetworkInterfaceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ContextualNetAddress::from(self.clone()).fmt(f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeConnectionConfigKind {
    Custom,
    Existing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NodeSettings {
    // pub connection_config_kind: NodeConnectionConfigKind,
    // pub existing_server: Server,
    pub rpc_kind: RpcKind,
    pub wrpc_url: String,
    pub wrpc_encoding: WrpcEncoding,
    pub enable_wrpc_json: bool,
    pub wrpc_json_network_interface: NetworkInterfaceConfig,
    pub enable_grpc: bool,
    pub grpc_network_interface: NetworkInterfaceConfig,
    pub enable_upnp: bool,

    pub network: Network,
    pub node_kind: KaspadNodeKind,
    pub kaspad_daemon_binary: String,
    pub kaspad_daemon_args: String,
    pub kaspad_daemon_args_enable: bool,
}

impl Default for NodeSettings {
    fn default() -> Self {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                let wrpc_url = "127.0.0.1";
            } else {
                use workflow_dom::utils::*;
                use workflow_core::runtime;
                let wrpc_url = if runtime::is_chrome_extension() {
                    "ws://127.0.0.1".to_string()
                } else {
                    let location = location().unwrap();
                    let hostname = location.hostname().expect("KaspadNodeKind: Unable to get hostname");
                    hostname.to_string()
                };
            }
        }

        Self {
            rpc_kind: RpcKind::Wrpc,
            wrpc_url: wrpc_url.to_string(),
            wrpc_encoding: WrpcEncoding::Borsh,
            enable_wrpc_json: false,
            wrpc_json_network_interface: NetworkInterfaceConfig::default(),
            enable_grpc: false,
            grpc_network_interface: NetworkInterfaceConfig::default(),
            enable_upnp: true,
            // network: Network::Mainnet,
            network: Network::Testnet10,
            // network: Network::default(),
            node_kind: KaspadNodeKind::default(),
            kaspad_daemon_binary: String::default(),
            kaspad_daemon_args: String::default(),
            kaspad_daemon_args_enable: false,
        }
    }
}

impl NodeSettings {
    cfg_if! {
        if #[cfg(not(target_arch = "wasm32"))] {
            #[allow(clippy::if_same_then_else)]
            pub fn compare(&self, other: &NodeSettings) -> Option<bool> {
                if self.network != other.network {
                    Some(true)
                } else if self.node_kind != other.node_kind {
                    Some(true)
                } else if self.enable_grpc != other.enable_grpc
                    || self.grpc_network_interface != other.grpc_network_interface
                    || self.wrpc_url != other.wrpc_url
                    || self.wrpc_encoding != other.wrpc_encoding
                    || self.enable_wrpc_json != other.enable_wrpc_json
                    || self.wrpc_json_network_interface != other.wrpc_json_network_interface
                    || self.enable_upnp != other.enable_upnp
                {
                    Some(self.node_kind != KaspadNodeKind::IntegratedInProc)
                } else if self.kaspad_daemon_args != other.kaspad_daemon_args
                    || self.kaspad_daemon_args_enable != other.kaspad_daemon_args_enable
                {
                    Some(self.node_kind.is_config_capable())
                } else if self.kaspad_daemon_binary != other.kaspad_daemon_binary {
                    Some(self.node_kind == KaspadNodeKind::ExternalAsDaemon)
                } else {
                    None
                }
            }
        } else {
            #[allow(clippy::if_same_then_else)]
            pub fn compare(&self, other: &NodeSettings) -> Option<bool> {
                if self.network != other.network {
                    Some(true)
                } else if self.node_kind != other.node_kind {
                    Some(true)
                } else if self.rpc_kind != other.rpc_kind
                    || self.wrpc_url != other.wrpc_url
                    || self.wrpc_encoding != other.wrpc_encoding
                {
                    Some(true)
                } else {
                    None
                }
            }

        }
    }
}

impl From<&NodeSettings> for RpcConfig {
    fn from(settings: &NodeSettings) -> Self {
        match settings.rpc_kind {
            RpcKind::Wrpc => RpcConfig::Wrpc {
                url: Some(settings.wrpc_url.clone()),
                encoding: settings.wrpc_encoding,
            },
            RpcKind::Grpc => RpcConfig::Grpc {
                url: Some(settings.grpc_network_interface.clone()),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MetricsSettings {
    pub graph_columns: usize,
    pub graph_height: usize,
    pub graph_range_from: isize,
    pub graph_range_to: isize,
    pub disabled: AHashSet<Metric>,
}

impl Default for MetricsSettings {
    fn default() -> Self {
        Self {
            graph_columns: 3,
            graph_height: 90,
            graph_range_from: -15 * 60,
            graph_range_to: 0,
            disabled: AHashSet::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UserInterfaceSettings {
    pub theme_color: String,
    pub theme_style: String,
    pub scale: f32,
    pub metrics: MetricsSettings,
    pub balance_padding: bool,
}

impl Default for UserInterfaceSettings {
    fn default() -> Self {
        Self {
            theme_color: "Dark".to_string(),
            theme_style: "Rounded".to_string(),
            scale: 1.0,
            metrics: MetricsSettings::default(),
            balance_padding: true,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeveloperSettings {
    pub enable: bool,
    pub enable_screen_capture: bool,
    pub disable_password_restrictions: bool,
    pub enable_experimental_features: bool,
    pub enable_custom_daemon_args: bool,
    pub market_monitor_on_testnet: bool,
}

impl Default for DeveloperSettings {
    fn default() -> Self {
        Self {
            enable: false,
            enable_screen_capture: true,
            disable_password_restrictions: false,
            enable_experimental_features: false,
            enable_custom_daemon_args: true,
            market_monitor_on_testnet: false,
        }
    }
}

impl DeveloperSettings {
    pub fn screen_capture_enabled(&self) -> bool {
        self.enable && self.enable_screen_capture
    }

    pub fn password_restrictions_disabled(&self) -> bool {
        self.enable && self.disable_password_restrictions
    }

    pub fn experimental_features_enabled(&self) -> bool {
        self.enable && self.enable_experimental_features
    }

    pub fn custom_daemon_args_enabled(&self) -> bool {
        self.enable && self.enable_custom_daemon_args
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    pub initialized: bool,
    pub splash_screen: bool,
    pub version: String,
    pub developer: DeveloperSettings,
    pub node: NodeSettings,
    pub user_interface: UserInterfaceSettings,
    pub language_code: String,
    pub update_monitor: bool,
    pub market_monitor: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            initialized: false,
            #[cfg(target_arch = "wasm32")]
            initialized: true,

            splash_screen: true,
            version: "0.0.0".to_string(),
            developer: DeveloperSettings::default(),
            node: NodeSettings::default(),
            user_interface: UserInterfaceSettings::default(),
            language_code: "en".to_string(),
            update_monitor: true,
            market_monitor: true,
        }
    }
}

impl Settings {}

fn storage() -> Result<Storage> {
    Ok(Storage::try_new("kaspa-ng.settings")?)
}

impl Settings {
    pub async fn store(&self) -> Result<()> {
        let storage = storage()?;
        storage.ensure_dir().await?;
        workflow_store::fs::write_json(storage.filename(), self).await?;
        Ok(())
    }

    pub fn store_sync(&self) -> Result<&Self> {
        let storage = storage()?;
        storage.ensure_dir_sync()?;
        workflow_store::fs::write_json_sync(storage.filename(), self)?;
        Ok(self)
    }

    pub async fn load() -> Result<Self> {
        use workflow_store::fs::read_json;

        let storage = storage()?;
        if storage.exists().await.unwrap_or(false) {
            match read_json::<Self>(storage.filename()).await {
                Ok(settings) => Ok(settings),
                Err(err) => {
                    log_warning!("Settings::load() error: {}", err);
                    Ok(Self::default())
                }
            }
        } else {
            Ok(Self::default())
        }
    }
}
