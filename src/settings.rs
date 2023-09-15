use crate::imports::*;
use kaspa_wallet_core::storage::local::storage::Storage;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "kebab-case")]
        pub enum KaspadNodeKind {
            Remote { url : String },
            #[default]
            InternalInProc,
            InternalAsDaemon,
            ExternalAsDaemon { path : String },
        }

    } else {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "kebab-case")]
        pub enum KaspadNodeKind {
            Remote { url : String },
        }

        impl Default for KaspadNodeKind {
            fn default() -> Self {
                use workflow_dom::utils::*;
                let url = window().location().hostname().expect("KaspadNodeKind: Unable to get hostname");
                KaspadNodeKind::Remote { url }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub url: String,
    pub network: Network,
    pub kaspad: KaspadNodeKind,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            url: "127.0.0.1".to_string(),
            // network: Network::Mainnet,
            network: Network::Testnet10,
            // kaspad_node: KaspadNodeKind::InternalInProc,
            kaspad: KaspadNodeKind::Remote {
                url: "".to_string(),
            },
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
