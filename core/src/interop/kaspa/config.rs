use crate::imports::*;
use kaspa_wrpc_server::address::WrpcNetAddress;

#[cfg(not(target_arch = "wasm32"))]
pub use kaspad_lib::args::Args;

#[derive(Debug)]
pub struct Config {
    network: Network,
}

impl From<NodeSettings> for Config {
    fn from(node_settings: NodeSettings) -> Self {
        let network = node_settings.network;
        Self { network }
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

        args.perf_metrics = true;
        args.perf_metrics_interval_sec = 1;
        args.yes = true;
        args.utxoindex = true;
        // args.rpclisten_borsh = Some(WrpcNetAddress::Default);

        args
    }
}
