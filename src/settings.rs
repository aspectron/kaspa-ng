use crate::imports::*;

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
