use crate::imports::*;
// use kaspa_wrpc_server::address::WrpcNetAddress;

#[cfg(not(target_arch = "wasm32"))]
pub use kaspad_lib::args::Args;

#[derive(Debug, Clone)]
pub struct Config {
    network: Network,
    enable_upnp: bool,
}

impl From<NodeSettings> for Config {
    fn from(node_settings: NodeSettings) -> Self {
        let network = node_settings.network;
        let enable_upnp = node_settings.enable_upnp;
        Self {
            network,
            enable_upnp,
        }
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
                args.disable_upnp = !config.enable_upnp;
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
                        args.push("--netsuffix=10");
                    }
                    Network::Testnet11 => {
                        args.push("--testnet");
                        args.push("--netsuffix=11");
                    }
                }

                args.push("--perf-metrics");
                args.push("--perf-metrics-interval-sec=1");
                args.push("--yes");
                args.push("--utxoindex");


                if !config.enable_upnp {
                    args.push("--disable-upnp");
                }

                // ---

                args.push("--rpclisten-borsh=default");

                args.into_iter().map(String::from).collect()
            }
        }

        impl IntoIterator for Config {
            type Item = String;
            type IntoIter = std::vec::IntoIter<Self::Item>;

            fn into_iter(self) -> Self::IntoIter {
                let args: Vec<String> = self.into();
                println!("CONFIG ARGS: {:?}", args);
                args.into_iter()
            }
        }
    }
}
