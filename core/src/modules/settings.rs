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

        // let theme = theme();

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

        #[allow(unused_variables)]
        let half_width = ui.ctx().screen_rect().width() * 0.5;
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
                                #[cfg(not(target_arch = "wasm32"))] {
                                    if !self.settings.developer_mode && matches!(*node_kind,KaspadNodeKind::IntegratedInProc|KaspadNodeKind::ExternalAsDaemon) {
                                        return;
                                    }
                                }
                                ui.radio_value(&mut self.settings.node.node_kind, *node_kind, node_kind.to_string());
                            });
                        });

                        match self.settings.node.node_kind {
                            KaspadNodeKind::Remote => {

                            },

                            #[cfg(not(target_arch = "wasm32"))]
                            KaspadNodeKind::IntegratedInProc => {
                                ui.horizontal_wrapped(|ui|{
                                    ui.set_max_width(half_width);
                                    ui.label(i18n("Please note that the integrated mode is experimental and does not currently show the sync progress information."));
                                });
                            },

                            #[cfg(not(target_arch = "wasm32"))]
                            KaspadNodeKind::ExternalAsDaemon => {
                                ui.horizontal(|ui|{
                                    ui.label(i18n("Rusty Kaspa Daemon Path:"));
                                    ui.add(TextEdit::singleline(&mut self.settings.node.kaspad_daemon_binary));
                                });
                                let path = std::path::PathBuf::from(&self.settings.node.kaspad_daemon_binary);
                                if path.exists() && !path.is_file() {
                                    ui.label(
                                        RichText::new(format!("Rusty Kaspa Daemon not found at '{path}'", path = self.settings.node.kaspad_daemon_binary))
                                            .color(theme().error_color),
                                    );
                                    node_settings_error = Some("Rusty Kaspa Daemon not found");
                                }
                            },
                            _ => { }
                        }

                    });

                if !self.grpc_network_interface.is_valid() {
                    node_settings_error = Some("Invalid gRPC Network Interface Configuration");
                } else {
                    self.settings.node.grpc_network_interface = self.grpc_network_interface.as_ref().try_into().unwrap(); //NetworkInterfaceConfig::try_from(&self.grpc_network_interface).unwrap();
                }
            });

            if self.settings.node.node_kind == KaspadNodeKind::Remote {
                CollapsingHeader::new("Remote p2p Node Configuration")
                    .default_open(true)
                    .show(ui, |ui| {

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

                        if let Err(err) = KaspaRpcClient::parse_url(self.settings.node.wrpc_url.clone(), self.settings.node.wrpc_encoding, self.settings.node.network.into()) {
                            ui.label(
                                RichText::new(format!("{err}"))
                                    .color(theme().warning_color),
                            );
                            node_settings_error = Some("Invalid wRPC URL");
                        }

                        #[cfg(not(target_arch = "wasm32"))]
                        ui.horizontal_wrapped(|ui|{
                            ui.set_max_width(half_width);
                            ui.label("Recommended arguments for the remote node:");
                            ui.code("kaspad --utxoindex --rpclisten-borsh=0.0.0.0");
                            ui.label("If you are running locally, use");
                            ui.code("--rpclisten-borsh=127.0.0.1.");
                        });




                    });
            }

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
                        .color(theme().warning_color),
                );
                ui.add_space(16.);
                ui.label(i18n("Unable to change node settings until the problem is resolved."));

                ui.add_space(16.);
                ui.separator();

            } else if node_settings_error.is_none() {
                if let Some(restart) = self.settings.node.compare(&core.settings.node) {

                    ui.add_space(16.);

                    if let Some(response) = ui.confirm_medium_apply_cancel(Align::Max) {
                        match response {
                            Confirm::Ack => {


                                core.settings = self.settings.clone();
                                core.settings.store_sync().unwrap();
                                if restart {
                                    // println!("NODE INTERFACE UPDATE: {:?}", self.settings.node);
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
            CollapsingHeader::new("Advanced")
                .default_open(false)
                .show(ui, |ui| {

                    ui.vertical(|ui|{
                        ui.checkbox(&mut self.settings.developer_mode, i18n("Developer Mode"));
                        ui.label("Developer mode enables experimental features");


                        ui.separator();
                        if ui.medium_button("Reset Settings").clicked() {
                            let settings = crate::settings::Settings::default();
                            settings.store_sync().unwrap();
                            #[cfg(target_arch = "wasm32")]
                            workflow_dom::utils::window().location().reload().ok();
                        }
                        
                    });
                });

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

