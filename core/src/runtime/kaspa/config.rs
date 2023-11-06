use crate::imports::*;
// use kaspa_wrpc_server::address::WrpcNetAddress;

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

cfg_if! {

    if #[cfg(not(target_arch = "wasm32"))] {
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

        impl From<Config> for Vec<String> {
            fn from(config: Config) -> Self {
                let mut args = Vec::new();

                match config.network {
                    Network::Mainnet => {}
                    Network::Testnet10 => {
                        args.push("--testnet");
                        args.push("--testnet-suffix=10");
                    }
                    Network::Testnet11 => {
                        args.push("--testnet");
                        args.push("--testnet-suffix=11");
                    }
                }

                args.push("--perf-metrics");
                args.push("--perf-metrics-interval-sec=1");
                args.push("--yes");
                args.push("--utxoindex");

                args.into_iter().map(String::from).collect()
            }
        }

        impl IntoIterator for Config {
            type Item = String;
            type IntoIter = std::vec::IntoIter<Self::Item>;

            fn into_iter(self) -> Self::IntoIter {
                let args: Vec<String> = self.into();
                args.into_iter()
            }
        }
    }
}
