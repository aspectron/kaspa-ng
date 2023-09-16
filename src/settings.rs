use crate::imports::*;
use kaspa_wallet_core::storage::local::storage::Storage;
use kaspa_wrpc_client::WrpcEncoding;
// use workflow_core::

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        #[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
        #[serde(rename_all = "kebab-case")]
        pub enum KaspadNodeKind {
            Remote,// { rpc_config : RpcConfig },
            #[default]
            InternalInProc,
            InternalAsDaemon,
            ExternalAsDaemon,// { path : String },
        }

    } else {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
        #[serde(rename_all = "kebab-case")]
        pub enum KaspadNodeKind {
            Remote,// { rpc_config : RpcConfig },
        }

        impl Default for KaspadNodeKind {
            fn default() -> Self {
                // use workflow_dom::utils::*;
                // let url = window().location().hostname().expect("KaspadNodeKind: Unable to get hostname");
                KaspadNodeKind::Remote// { rpc_config : RpcConfig::default() }
            }
        }
    }
}


#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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
pub struct Settings {
    // #[serde(rename = "rpc")]
    pub rpc_kind: RpcKind,
    pub wrpc_url : String,
    pub wrpc_encoding : WrpcEncoding,
    pub grpc_url : String,

    // pub rpc: RpcConfig,
    pub network: Network,
    pub kaspad: KaspadNodeKind,
    pub kaspad_node_binary: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {

        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                let wrpc_url = "127.0.0.1";
            } else {
                use workflow_dom::utils::*;
                let wrpc_url = window().location().hostname().expect("KaspadNodeKind: Unable to get hostname");
            }
        }

        Self {

            wrpc_url : wrpc_url.to_string(),  // : "127.0.0.1".to_string(),
            wrpc_encoding : WrpcEncoding::Borsh,
            grpc_url : "127.0.0.1".to_string(),
            rpc_kind : RpcKind::Wrpc,
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

impl From<&Settings> for RpcConfig {
    fn from(settings: &Settings) -> Self {
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

