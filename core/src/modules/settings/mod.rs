
use crate::imports::*;

pub struct Settings {
    #[allow(dead_code)]
    runtime: Runtime,
    settings : crate::settings::Settings,
    grpc_network_interface : NetworkInterfaceEditor,
    reset_settings : bool,


}

impl Settings {
    pub fn new(runtime: Runtime) -> Self {
        Self { 
            runtime,
            settings : crate::settings::Settings::default(),
            grpc_network_interface : NetworkInterfaceEditor::default(),
            reset_settings : false,
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
        ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                self.render_settings(core,ui);
            });
    }
}

impl Settings {

    fn render_settings(
        &mut self,
        core: &mut Core,
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
                                    if !core.settings.developer.enable_experimental_features() && matches!(*node_kind,KaspadNodeKind::IntegratedInProc|KaspadNodeKind::ExternalAsDaemon) {
                                        return;
                                    }
                                }
                                ui.radio_value(&mut self.settings.node.node_kind, *node_kind, node_kind.to_string()).on_hover_text_at_pointer(node_kind.describe());
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

                                // let binary_path = self.settings.node.kaspad_daemon_binary.clone();

                                ui.horizontal(|ui|{
                                    ui.label(i18n("Rusty Kaspa Daemon Path:"));
                                    ui.add(TextEdit::singleline(&mut self.settings.node.kaspad_daemon_binary));
                                });

                                // if binary_path != self.settings.node.kaspad_daemon_binary {
                                    let path = std::path::PathBuf::from(&self.settings.node.kaspad_daemon_binary);
                                    if path.exists() && !path.is_file() {
                                        ui.label(
                                            RichText::new(format!("Rusty Kaspa Daemon not found at '{path}'", path = self.settings.node.kaspad_daemon_binary))
                                                .color(theme_color().error_color),
                                        );
                                        node_settings_error = Some("Rusty Kaspa Daemon not found");
                                    }
                                // }
                            },
                            _ => { }
                        }

                        #[cfg(not(target_arch = "wasm32"))]
                        if core.settings.developer.enable_custom_daemon_args() && core.settings.node.node_kind.is_config_capable() {
                            use kaspad_lib::args::Args;
                            use clap::error::ErrorKind as ClapErrorKind;
                            use crate::runtime::services::kaspa::Config;

                            ui.add_space(4.);
                            ui.checkbox(&mut self.settings.node.kaspad_daemon_args_enable, i18n("Enable custom daemon arguments"));
                            ui.add_space(4.);

                            if self.settings.node.kaspad_daemon_args_enable {
                                
                                ui.vertical(|ui| {
                                    ui.label("Resulting daemon arguments:");
                                    ui.add_space(4.);

                                    let config = Config::from(self.settings.node.clone());
                                    let config = Vec::<String>::from(config).join(" ");
                                    ui.label(RichText::new(config).code().font(FontId::monospace(14.0)).color(theme_color().strong_color));
                                    ui.add_space(4.);


                                    ui.label("Custom arguments:");
                                    let width = ui.available_width() * 0.4;
                                    let height = 48.0;
                                    ui.add_sized(vec2(width,height),TextEdit::multiline(&mut self.settings.node.kaspad_daemon_args).code_editor().font(FontId::monospace(14.0)));
                                    ui.add_space(4.);
                                });

                                let args = format!("kaspad {}",self.settings.node.kaspad_daemon_args.trim());
                                let args = args.trim().split(' ').collect::<Vec<&str>>();
                                match Args::parse(args.iter()) {
                                    Ok(_) => { },
                                    Err(err) => {

                                        if matches!(err.kind(), ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion) {
                                            ui.label(
                                                RichText::new("--help and --version are not allowed")
                                                    .color(theme_color().warning_color),
                                            );
                                        } else {
                                            let help = err.to_string();
                                            let lines = help.split('\n').collect::<Vec<&str>>();
                                            let text = if let Some(idx) = lines.iter().position(|line| line.starts_with("For more info") || line.starts_with("Usage:")) {
                                                lines[0..idx].join("\n")
                                            } else {
                                                lines.join("\n")
                                            };

                                            ui.label(
                                                RichText::new(text.trim())
                                                    .color(theme_color().warning_color),
                                            );
                                        }
                                        ui.add_space(4.);
                                        node_settings_error = Some("Invalid daemon arguments");
                                    }
                                }
                            }
                        }

                    });

                if !self.grpc_network_interface.is_valid() {
                    node_settings_error = Some("Invalid gRPC network interface configuration");
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
                                    .color(theme_color().warning_color),
                            );
                            node_settings_error = Some("Invalid wRPC URL");
                        }

                        #[cfg(not(target_arch = "wasm32"))]
                        ui.horizontal_wrapped(|ui|{
                            ui.set_max_width(half_width);
                            ui.label("Recommended arguments for the remote node:");
                            ui.label(RichText::new("kaspad --utxoindex --rpclisten-borsh=0.0.0.0").code().font(FontId::monospace(14.0)).color(theme_color().strong_color));
                            ui.label("If you are running locally, use");
                            ui.label(RichText::new("--rpclisten-borsh=127.0.0.1.").code().font(FontId::monospace(14.0)).color(theme_color().strong_color));
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
                ui.add_space(4.);
                ui.label(
                    RichText::new(error.to_string())
                        .color(theme_color().error_color),
                );
                ui.add_space(4.);
                ui.label(i18n("Unable to change node settings until the problem is resolved."));

                ui.add_space(8.);
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
                                    self.runtime.kaspa_service().update_services(&self.settings.node);
                                }
                            },
                            Confirm::Nack => {
                                self.settings = core.settings.clone();
                                self.grpc_network_interface = NetworkInterfaceEditor::try_from(&self.settings.node.grpc_network_interface).unwrap();
                            }
                        }
                    }
                    ui.separator();
                }
            }

            CollapsingHeader::new("Centralized Services")
                .default_open(true)
                .show(ui, |ui| {

                    CollapsingHeader::new("Market Monitor")
                        .default_open(true)
                        .show(ui, |ui| {
                            let mut v = false;
                            if ui.checkbox(&mut v, i18n("Enable Market Monitor")).changed() {
                            }
                        });

                    #[cfg(not(target_arch = "wasm32"))]
                    CollapsingHeader::new("Check for Updates")
                        .default_open(true)
                        .show(ui, |ui| {
                            let mut v = false;
                            if ui.checkbox(&mut v, i18n("Check for Software Updates via GitHub")).changed() {
                            }
        
                        });    
                });

            CollapsingHeader::new("Advanced")
                .default_open(false)
                .show(ui, |ui| {

                    ui.vertical(|ui|{
                        ui.checkbox(&mut self.settings.developer.enable, i18n("Developer Mode"));
                        ui.label("Developer mode enables advanced and experimental features");
                    });

                    ui.vertical(|ui|{
                        if self.settings.developer.enable {
                            #[cfg(not(target_arch = "wasm32"))]
                            ui.checkbox(
                                &mut self.settings.developer.enable_experimental_features, 
                                i18n("Enable experimental features")
                            ).on_hover_text_at_pointer(
                                i18n("Enables features currently in development")
                            );
                            
                            #[cfg(not(target_arch = "wasm32"))]
                            ui.checkbox(
                                &mut self.settings.developer.enable_custom_daemon_args, 
                                i18n("Allow custom daemon arguments")
                            ).on_hover_text_at_pointer(
                                i18n("Allow custom arguments for the Rusty Kaspa daemon")
                            );
                            
                            ui.checkbox(
                                &mut self.settings.developer.disable_password_restrictions, 
                                i18n("Disable password score restrictions")
                            ).on_hover_text_at_pointer(
                                i18n("Removes security restrictions, allows for single-letter passwords")
                            );

                            #[cfg(not(target_arch = "wasm32"))]
                            ui.checkbox(
                                &mut self.settings.developer.enable_screen_capture, 
                                i18n("Screen capture")
                            ).on_hover_text_at_pointer(
                                i18n("Allows you to take screenshots from within the application")
                            );
                        }
                    });

                    if self.settings.developer != core.settings.developer {
                        ui.add_space(16.);
                        if let Some(response) = ui.confirm_medium_apply_cancel(Align::Max) {
                            match response {
                                Confirm::Ack => {
                                    core.settings.developer = self.settings.developer.clone();
                                    core.settings.store_sync().unwrap();
                                },
                                Confirm::Nack => {
                                    self.settings.developer = core.settings.developer.clone();
                                }
                            }
                        }
                        ui.separator();
                    }

                    if !self.reset_settings {
                        ui.vertical(|ui|{
                            if self.settings.developer == core.settings.developer {
                                ui.set_max_width(340.);
                                ui.separator();
                            }
                            if ui.medium_button("Reset Settings").clicked() {
                                self.reset_settings = true;
                            }
                        });
                    } else {
                        ui.add_space(16.);
                        ui.label(RichText::new(i18n("Are you sure you want to reset all settings?")).color(theme_color().warning_color));
                        ui.add_space(16.);
                        if let Some(response) = ui.confirm_medium_apply_cancel(Align::Min) {
                            match response {
                                Confirm::Ack => {
                                    let settings = crate::settings::Settings {
                                        initialized : true,
                                        ..Default::default()
                                    };
                                    self.settings = settings.clone();
                                    settings.store_sync().unwrap();
                                    #[cfg(target_arch = "wasm32")]
                                    workflow_dom::utils::window().location().reload().ok();
                                },
                                Confirm::Nack => {
                                    self.reset_settings = false;
                                }
                            }
                        }
                        ui.separator();
                    }



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

