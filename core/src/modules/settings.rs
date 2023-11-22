use crate::imports::*;


// pub struct Config {
//     network: Network,
// }

pub struct Settings {
    #[allow(dead_code)]
    runtime: Runtime,
    settings : crate::settings::Settings,
    grpc_network_interface : NetworkInterfaceEditor,


}

impl Settings {
    pub fn new(runtime: Runtime) -> Self {
        Self { 
            runtime,
            settings : crate::settings::Settings::default(),
            grpc_network_interface : NetworkInterfaceEditor::default(),
        }
    }

    pub fn load(&mut self, settings : crate::settings::Settings) {
        self.settings = settings;

        self.grpc_network_interface = NetworkInterfaceEditor::try_from(&self.settings.node.grpc_network_interface).unwrap();
    }
}

impl ModuleT for Settings {

    fn init(&mut self, wallet : &mut Core) {
        self.load(wallet.settings.clone());
    }

    fn style(&self) -> ModuleStyle {
        // ModuleStyle::Large
        ModuleStyle::Default
    }


    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        let theme = theme();

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

        let mut node_settings_error = None;

        CollapsingHeader::new("Kaspa p2p Network & Node Connection")
            .default_open(true)
            .show(ui, |ui| {

                CollapsingHeader::new("Kaspa Network")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui|{
                            Network::iter().for_each(|network| {
                                ui.radio_value(&mut self.settings.node.network, *network, network.to_string());
                            });
                        });
                    });

                CollapsingHeader::new("Kaspa Node")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui|{
                            KaspadNodeKind::iter().for_each(|node_kind| {
                                ui.radio_value(&mut self.settings.node.node_kind, *node_kind, node_kind.to_string());
                            });
                        });


                        if self.settings.node.node_kind == KaspadNodeKind::Remote {



                            // RANDOM, COMMUNITY NODES, CUSTOM
                            // let mut wrpc_url = self.settings.node.wrpc_url.clone();
                            ui.horizontal(|ui|{
                                ui.label(i18n("wRPC Encoding:"));
                                WrpcEncoding::iter().for_each(|encoding| {
                                    ui.radio_value(&mut self.settings.node.wrpc_encoding, *encoding, encoding.to_string());
                                });
                            });


                            ui.horizontal(|ui|{
                                ui.label(i18n("wRPC URL:"));
                                ui.add(TextEdit::singleline(&mut self.settings.node.wrpc_url));
                                
                            });

                            if let Err(err) = KaspaRpcClient::parse_url(Some(self.settings.node.wrpc_url.clone()), self.settings.node.wrpc_encoding, self.settings.node.network.into()) {
                                ui.label(
                                    RichText::new(format!("{err}"))
                                        .color(theme.warning_color),
                                );
                                node_settings_error = Some("Invalid wRPC URL");
                                // return;
                            }

                            #[cfg(not(target_arch = "wasm32"))]
                            ui.horizontal_wrapped(|ui|{
                                ui.label("Recommended arguments for the remote node:");
                                ui.code("kaspad --utxoindex --rpclisten-borsh=0.0.0.0");
                                ui.label("If you are running locally, use");
                                ui.code("--rpclisten-borsh=127.0.0.1.");
                            });
    

                        }
                    });

                if !self.grpc_network_interface.is_valid() {
                    node_settings_error = Some("Invalid gRPC Network Interface Configuration");
                } else {
                    self.settings.node.grpc_network_interface = self.grpc_network_interface.as_ref().try_into().unwrap(); //NetworkInterfaceConfig::try_from(&self.grpc_network_interface).unwrap();
                }
            });


            #[cfg(not(target_arch = "wasm32"))]
            if self.settings.node.node_kind.is_config_capable() {

                CollapsingHeader::new("Local p2p Node Configuration")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.vertical(|ui|{
                            CollapsingHeader::new("Client RPC")
                                .default_open(true)
                                .show(ui, |ui| {
                                    ui.vertical(|ui|{

                                        ui.checkbox(&mut self.settings.node.enable_grpc, i18n("Enable gRPC"));
                                        if self.settings.node.enable_grpc {

                                            CollapsingHeader::new("gRPC Network Interface & Port")
                                                .default_open(true)
                                                .show(ui, |ui| {
                                                    self.grpc_network_interface.ui(ui);
                                                });
                                            // - TODO
                                            // ui.add(TextEdit::singleline(&mut self.settings.node.grpc_network_interface));
                                        }
                                    });

                            });
                        // });
                        
                            CollapsingHeader::new("p2p RPC")
                                .default_open(true)
                                .show(ui, |ui| {
                                    ui.vertical(|ui|{
                                        ui.checkbox(&mut self.settings.node.enable_upnp, i18n("Enable UPnP"));
                                    });
                                });
                            });
                    });
            } // is_config_capable

            if let Some(error) = node_settings_error {
                ui.label(
                    RichText::new(error.to_string())
                        .color(theme.warning_color),
                );

                return;
            }

            if node_settings_error.is_none() {
                if let Some(restart) = self.settings.node.compare(&core.settings.node) {

                    if let Some(response) = ui.confirm_medium_apply_cancel(Align::Max) {
                        match response {
                            Confirm::Ack => {


                                core.settings = self.settings.clone();
                                core.settings.store_sync().unwrap();
                                if restart {
                                    println!("NODE INTERFACE UPDATE: {:?}", self.settings.node);
                                    self.runtime.kaspa_service().update_services(&self.settings.node);
                                    // println!("TODO - restart");
                                }
                            },
                            Confirm::Nack => {
                                self.settings = core.settings.clone();
                            }
                        }
                    }

                    ui.separator();
                }
            }

            // ----------------------------

            if ui.button("Reset Settings").clicked() {
                let settings = crate::settings::Settings::default();
                settings.store_sync().unwrap();
                #[cfg(target_arch = "wasm32")]
                workflow_dom::utils::window().location().reload().ok();
            }

            // if ui.button("Test Toast").clicked() {
            //     self.runtime.try_send(Events::Notify {
            //         notification : UserNotification::info("Test Toast")
            //     }).unwrap();
            // }
            // ui.add_space(32.);
            // if ui.button("Test Panic").clicked() {
            //     panic!("Testing panic...");
            // }
    }
}

// if let Some(result) = spawn!(async move {

//     println!("Spawn executing...");
//     Ok(123)
// }) {

//     println!("Result {:?}", result);
//     ui.label(format!("Result {:?}", result));
// }
