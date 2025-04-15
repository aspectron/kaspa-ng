use crate::imports::*;
use kaspa_addresses::Prefix as AddressPrefix;
use kaspa_consensus_core::config::params::Params;
use kaspa_wallet_core::utxo::NetworkParams;

pub const BASIC_TRANSACTION_MASS: u64 = 2036;

#[derive(
    Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(rename_all = "kebab-case")]
pub enum Network {
    #[default]
    Mainnet,
    #[serde(alias = "testnet-10")]
    Testnet10,
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Testnet10 => write!(f, "testnet-10"),
        }
    }
}

impl FromStr for Network {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "mainnet" => Ok(Network::Mainnet),
            "testnet-10" => Ok(Network::Testnet10),
            _ => Err(Error::InvalidNetwork(s.to_string())),
        }
    }
}

impl From<Network> for NetworkType {
    fn from(network: Network) -> Self {
        match network {
            Network::Mainnet => NetworkType::Mainnet,
            Network::Testnet10 => NetworkType::Testnet,
        }
    }
}

impl From<&Network> for NetworkType {
    fn from(network: &Network) -> Self {
        match network {
            Network::Mainnet => NetworkType::Mainnet,
            Network::Testnet10 => NetworkType::Testnet,
        }
    }
}

impl From<Network> for NetworkId {
    fn from(network: Network) -> Self {
        match network {
            Network::Mainnet => NetworkId::new(network.into()),
            Network::Testnet10 => NetworkId::with_suffix(network.into(), 10),
        }
    }
}

impl From<&Network> for AddressPrefix {
    fn from(network: &Network) -> Self {
        NetworkType::from(network).into()
    }
}

impl From<Network> for AddressPrefix {
    fn from(network: Network) -> Self {
        NetworkType::from(network).into()
    }
}

impl From<&Network> for NetworkId {
    fn from(network: &Network) -> Self {
        match network {
            Network::Mainnet => NetworkId::new(network.into()),
            Network::Testnet10 => NetworkId::with_suffix(network.into(), 10),
        }
    }
}

impl From<NetworkId> for Network {
    fn from(value: NetworkId) -> Self {
        match value.network_type {
            NetworkType::Mainnet => Network::Mainnet,
            NetworkType::Testnet => match value.suffix {
                Some(10) => Network::Testnet10,
                Some(x) => unreachable!("Testnet suffix {} is not supported", x),
                None => panic!("Testnet suffix not provided"),
            },
            NetworkType::Devnet => unreachable!("Devnet is not supported"),
            NetworkType::Simnet => unreachable!("Simnet is not supported"),
        }
    }
}

impl From<Network> for Params {
    fn from(network: Network) -> Self {
        NetworkId::from(network).into()
    }
}

impl From<&Network> for Params {
    fn from(network: &Network) -> Self {
        NetworkId::from(network).into()
    }
}

impl From<Network> for &'static NetworkParams {
    fn from(network: Network) -> Self {
        NetworkParams::from(NetworkId::from(network))
    }
}

impl From<&Network> for &'static NetworkParams {
    fn from(network: &Network) -> Self {
        NetworkParams::from(NetworkId::from(network))
    }
}

const NETWORKS: [Network; 2] = [Network::Mainnet, Network::Testnet10];

impl Network {
    pub fn iter() -> impl Iterator<Item = &'static Network> {
        NETWORKS.iter()
    }

    pub fn name(&self) -> &str {
        match self {
            Network::Mainnet => i18n("Mainnet"),
            Network::Testnet10 => i18n("Testnet 10"),
        }
    }

    pub fn describe(&self) -> &str {
        match self {
            Network::Mainnet => i18n("Main Kaspa network"),
            Network::Testnet10 => i18n("10 BPS test network"),
        }
    }

    pub fn tps(&self) -> u64 {
        let params = Params::from(*self);
        // TODO: use DAA score to determine the correct BPS value
        params.max_block_mass / BASIC_TRANSACTION_MASS * params.bps().after()
    }
}

const MAX_NETWORK_PRESSURE_SAMPLES: usize = 16;
const NETWORK_PRESSURE_ALPHA_HIGH: f32 = 0.8;
const NETWORK_PRESSURE_ALPHA_LOW: f32 = 0.5;
const NETWORK_PRESSURE_THRESHOLD_HIGH: f32 = 0.4;
const NETWORK_PRESSURE_THRESHOLD_LOW: f32 = 0.2;
const NETWORK_CAPACITY_THRESHOLD: usize = 90;

#[derive(Default, Debug, Clone)]
pub struct NetworkPressure {
    pub network_pressure_samples: VecDeque<f32>,
    pub pressure: f32,
    pub is_high: bool,
}

impl NetworkPressure {
    pub fn clear(&mut self) {
        self.network_pressure_samples.clear();
        self.pressure = 0.0;
    }

    fn insert_sample(&mut self, pressure: f32, alpha: f32) {
        let pressure = alpha * pressure + (1.0 - alpha) * self.pressure;
        self.network_pressure_samples.push_back(pressure);
        if self.network_pressure_samples.len() > MAX_NETWORK_PRESSURE_SAMPLES {
            self.network_pressure_samples.pop_front();
        }
    }

    pub fn update_mempool_size(&mut self, mempool_size: usize, network: &Network) {
        let pressure = mempool_size as f32 / network.tps() as f32;

        if pressure > self.pressure {
            self.insert_sample(pressure, NETWORK_PRESSURE_ALPHA_HIGH);
        } else {
            self.insert_sample(pressure, NETWORK_PRESSURE_ALPHA_LOW);
        }

        let average_pressure = self.network_pressure_samples.iter().sum::<f32>()
            / self.network_pressure_samples.len() as f32;

        self.pressure = average_pressure;
        if self.is_high {
            self.is_high = self.pressure > NETWORK_PRESSURE_THRESHOLD_LOW;
        } else {
            self.is_high = self.pressure > NETWORK_PRESSURE_THRESHOLD_HIGH;
        }

        // println!("{:?}", self.network_pressure_samples);
        // println!("mempool: {} capacity: {}% avg pressure: {} is high: {}", mempool_size, self.capacity(), self.pressure, self.is_high());
    }

    pub fn is_high(&self) -> bool {
        self.is_high
    }

    pub fn pressure(&self) -> f32 {
        self.pressure
    }

    pub fn capacity(&self) -> usize {
        (self.pressure * 100.0).min(100.0) as usize
    }

    pub fn above_capacity(&self) -> bool {
        self.capacity() > NETWORK_CAPACITY_THRESHOLD
    }

    pub fn below_capacity(&self) -> bool {
        self.capacity() <= NETWORK_CAPACITY_THRESHOLD
    }
}
