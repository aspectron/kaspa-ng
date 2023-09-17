use crate::imports::*;


// pub struct Config {
//     network: Network,
// }

pub struct Settings {
    #[allow(dead_code)]
    interop: Interop,
    settings : crate::settings::Settings,
    // pub kaspad: KaspadNodeKind,

}

impl Settings {
    pub fn new(interop: Interop) -> Self {
        Self { interop, settings : crate::settings::Settings::default() }
    }

}

impl ModuleT for Settings {

    fn init(&mut self, wallet : &mut Wallet) {
        self.settings = wallet.settings.clone();
    }

    fn render(
        &mut self,
        wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        // wallet.style()

        ui.style_mut().text_styles = wallet.default_style.text_styles.clone();

        // ui.heading("Settings");
        // ui.separator();
        // ui.label("This is the settings page");

        // - wRPC JSON, wRPC BORSH, gRPC
        // - PUBLIC RPC LISTEN !
        // - MAINNET, TESTNET-10, TESTNET-11
        // - CUSTOM RPC PORTS
        // - EXTERNAL IP
        // -
        // ---
        // - pub connect_peers: Vec<ContextualNetAddress>,
        // - pub add_peers: Vec<ContextualNetAddress>,
        // - pub outbound_target: usize,
        // - pub inbound_limit: usize,
        // - pub rpc_max_clients: usize, gRPC
        // - pub enable_unsynced_mining: bool,
        // - pub enable_mainnet_mining: bool,
        // - pub perf_metrics: bool,
        // - pub perf_metrics_interval_sec: u64,

        CollapsingHeader::new("Kaspa p2p Node & Connection")
            .default_open(true)
            .show(ui, |ui| {

                CollapsingHeader::new("Kaspa Network")
                    .default_open(true)
                    .show(ui, |ui| {
                        // ui.label("This is the settings page");

                        ui.horizontal_wrapped(|ui|{
                            ui.radio_value(&mut self.settings.network, Network::Mainnet, "MAINNET");
                            ui.radio_value(&mut self.settings.network, Network::Testnet10, "TESTNET-10");
                            ui.radio_value(&mut self.settings.network, Network::Testnet11, "TESTNET-11");
                        });

                        // if let Some(result) = spawn!(async move {

                        //     println!("Spawn executing...");
                        //     Ok(123)
                        // }) {

                        //     println!("Result {:?}", result);
                        //     ui.label(format!("Result {:?}", result));
                        // }

                    });


                CollapsingHeader::new("Kaspa Node")
                    .default_open(true)
                    .show(ui, |ui| {

                    // ui.label

                        ui.horizontal_wrapped(|ui|{

                            ui.radio_value(&mut self.settings.kaspad, KaspadNodeKind::Remote, "Remote");
                            cfg_if! {
                                if #[cfg(not(target_arch = "wasm32"))] {
                                    ui.radio_value(&mut self.settings.kaspad, KaspadNodeKind::InternalInProc, "Internal");
                                    ui.radio_value(&mut self.settings.kaspad, KaspadNodeKind::InternalAsDaemon, "Internal Daemon");
                                    ui.radio_value(&mut self.settings.kaspad, KaspadNodeKind::ExternalAsDaemon, "External Daemon");
                                }
                            }
                        });

                        // ui.label("")
                            
                        if self.settings.kaspad != wallet.settings.kaspad && ui.button("Apply").clicked() {
                            wallet.settings.kaspad = self.settings.kaspad;
                            wallet.settings.store().unwrap();
                        }

                        ui.label("This is the settings page");

                    });

                CollapsingHeader::new("RPC Protocol")
                    .default_open(true)
                    .show(ui, |ui| {

                        ui.horizontal(|ui|{
                            ui.label("URL: ");
                            ui.add(TextEdit::singleline(&mut self.settings.wrpc_url));
                        });
                        ui.horizontal_wrapped(|_ui|{
                            // ui.radio_value(&mut );
                        });

                        ui.label("This is the settings page");

                    });
            });
    }
}
