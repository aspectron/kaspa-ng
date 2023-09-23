use crate::imports::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Network {
    Mainnet,
    Testnet10,
    Testnet11,
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Testnet10 => write!(f, "testnet-10"),
            Network::Testnet11 => write!(f, "testnet-11"),
        }
    }
}

impl From<Network> for NetworkType {
    fn from(network: Network) -> Self {
        match network {
            Network::Mainnet => NetworkType::Mainnet,
            Network::Testnet10 => NetworkType::Testnet,
            Network::Testnet11 => NetworkType::Testnet,
        }
    }
}

impl From<Network> for NetworkId {
    fn from(network: Network) -> Self {
        match network {
            Network::Mainnet => NetworkId::new(network.into()),
            Network::Testnet10 => NetworkId::with_suffix(network.into(), 10),
            Network::Testnet11 => NetworkId::with_suffix(network.into(), 11),
        }
    }
}

const NETWORKS: [Network; 3] = [Network::Mainnet, Network::Testnet10, Network::Testnet11];

impl Network {
    pub fn iter() -> impl Iterator<Item = &'static Network> {
        NETWORKS.iter()
    }
}
