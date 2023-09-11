use crate::imports::*;
use kaspa_wrpc_server::address::WrpcNetAddress;
pub use kaspad::args::Args;

pub struct Config {
    network: Network,
}

impl From<Settings> for Config {
    fn from(settings: Settings) -> Self {
        let network = settings.network;
        Self { network }
    }
}

impl From<Config> for Args {
    fn from(config: Config) -> Self {
        let mut args = Args::default();
        match config.network {
            Network::Mainnet => {}
            Network::Testnet10 => {
                args.testnet = true;
                args.testnet_suffix = 10;
            }
            Network::Testnet11 => {
                args.testnet = true;
                args.testnet_suffix = 11;
            }
        }

        args.utxoindex = true;
        args.rpclisten_borsh = Some(WrpcNetAddress::Default);

        args
    }
}
