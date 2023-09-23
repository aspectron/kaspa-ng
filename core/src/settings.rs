use crate::imports::*;
use kaspa_wallet_core::storage::local::storage::Storage;
use kaspa_wrpc_client::WrpcEncoding;
// use workflow_core::

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        #[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
        #[serde(rename_all = "kebab-case")]
        pub enum KaspadNodeKind {
            Remote,
            #[default]
            IntegratedInProc,
            IntegratedAsDaemon,
            ExternalAsDaemon,
        }

        const KASPAD_NODE_KINDS: [KaspadNodeKind; 4] = [
            KaspadNodeKind::Remote,
            KaspadNodeKind::IntegratedInProc,
            KaspadNodeKind::IntegratedAsDaemon,
            KaspadNodeKind::ExternalAsDaemon,
        ];

        impl std::fmt::Display for KaspadNodeKind {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    KaspadNodeKind::Remote => write!(f, "Remote"),
                    KaspadNodeKind::IntegratedInProc => write!(f, "Integrated"),
                    KaspadNodeKind::IntegratedAsDaemon => write!(f, "Integrated Daemon"),
                    KaspadNodeKind::ExternalAsDaemon => write!(f, "External Daemon"),
                }
            }
        }

    } else {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
        #[serde(rename_all = "kebab-case")]
        pub enum KaspadNodeKind {
            Remote,// { rpc_config : RpcConfig },
        }

        impl Default for KaspadNodeKind {
            fn default() -> Self {
                KaspadNodeKind::Remote
            }
        }

        const KASPAD_NODE_KINDS: [KaspadNodeKind; 1] = [
            KaspadNodeKind::Remote,
        ];

        impl std::fmt::Display for KaspadNodeKind {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
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
    // #[default]
    // Wrpc,
    // Grpc,
    Wrpc { url: String, encoding: WrpcEncoding },
    Grpc { url: String },
}

// impl Default for RpcConfig {
//     fn default() -> Self {
//         cfg_if! {
//             if #[cfg(not(target_arch = "wasm32"))] {
//                 let url = "127.0.0.1";
//             } else {
//                 use workflow_dom::utils::*;
//                 let url = window().location().hostname().expect("KaspadNodeKind: Unable to get hostname");
//             }
//         }
//         RpcConfig::WRPC {
//             url: url.to_string(),
//             encoding: WrpcEncoding::Borsh,
//         }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NodeSettings {
    pub rpc_kind: RpcKind,
    pub wrpc_url: String,
    pub wrpc_encoding: WrpcEncoding,
    pub grpc_url: String,

    pub network: Network,
    pub kaspad: KaspadNodeKind,
    pub kaspad_node_binary: Option<String>,

}


impl Default for NodeSettings {
    fn default() -> Self {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                let wrpc_url = "127.0.0.1";
                // let wrpc_url = "ws://127.0.0.1:17210".to_string();
            } else {
                use workflow_dom::utils::*;

                let location = window().location();
                let protocol = location.protocol().expect("unable to get protocol");
                let hostname = window().location().hostname().expect("KaspadNodeKind: Unable to get hostname");
                log_warning!("protocol: {}", protocol);
                log_warning!("hostname: {}", hostname);
                let wrpc_url = if protocol == "chrome-extension:" {
                    "ws://127.0.0.1".to_string()
                } else {
                    hostname.to_string()
                };

                //,Network::Testnet10.default_borsh_rpc_port()); // window().location().hostname().expect("KaspadNodeKind: Unable to get hostname");
            }
        }

        Self {
            wrpc_url: wrpc_url.to_string(), // : "127.0.0.1".to_string(),
            wrpc_encoding: WrpcEncoding::Borsh,
            grpc_url: "127.0.0.1".to_string(),
            rpc_kind: RpcKind::Wrpc,
            // rpc: RpcConfig::default(),
            // network: Network::Mainnet,
            network: Network::Testnet10,
            // kaspad_node: KaspadNodeKind::InternalInProc,
            kaspad: KaspadNodeKind::Remote,
            kaspad_node_binary: None,
            //  {
            //     url: "".to_string(),
            // },
        }
    }
}

impl NodeSettings {
    cfg_if! {
        if #[cfg(not(target_arch = "wasm32"))] {
            pub fn compare(&self, other: &NodeSettings) -> Option<bool> {
                if self.network != other.network {
                    Some(true)
                } else if self.kaspad != other.kaspad {
                    Some(true)
                } else if self.rpc_kind != other.rpc_kind
                    || self.wrpc_url != other.wrpc_url
                    || self.wrpc_encoding != other.wrpc_encoding
                    || self.grpc_url != other.grpc_url
                {
                    Some(self.kaspad != KaspadNodeKind::IntegratedInProc)
                } else if self.kaspad_node_binary != other.kaspad_node_binary {
                    Some(self.kaspad == KaspadNodeKind::ExternalAsDaemon)
                } else {
                    None
                }
            }
        } else {
            pub fn compare(&self, other: &NodeSettings) -> Option<bool> {
                if self.network != other.network {
                    Some(true)
                } else if self.kaspad != other.kaspad {
                    Some(true)
                } else if self.rpc_kind != other.rpc_kind
                    || self.wrpc_url != other.wrpc_url
                    || self.wrpc_encoding != other.wrpc_encoding
                    || self.grpc_url != other.grpc_url
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
                url: settings.wrpc_url.clone(),
                encoding: settings.wrpc_encoding,
            },
            RpcKind::Grpc => RpcConfig::Grpc {
                url: settings.grpc_url.clone(),
            },
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UxSettings {

}

impl Default for UxSettings {
    fn default() -> Self {
        Self {
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    pub node : NodeSettings,
    pub ux : UxSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            node: NodeSettings::default(),
            ux: UxSettings::default(),
        }
    }
}

impl Settings {
    // Returns `Option<bool>` here `Option` indicates that 
    // settings have changed and `bool` indicates if the change
    // requires the node subsystem restart.
 
}


fn storage() -> Result<Storage> {
    Ok(Storage::try_new("kaspa-egui")?)
}

impl Settings {
    pub fn store(&self) -> Result<()> {
        let storage = storage()?;
        storage.ensure_dir_sync()?;
        workflow_store::fs::write_json_sync(storage.filename(), self)?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        use workflow_store::fs::read_json_sync;

        let storage = storage()?;
        if storage.exists_sync().unwrap_or(false) {
            Ok(Self::default())
        } else {
            match read_json_sync::<Self>(storage.filename()) {
                Ok(settings) => Ok(settings),
                Err(err) => {
                    log_error!("Settings::load: {}", err);
                    Ok(Self::default())
                }
            }
        }
    }
}

