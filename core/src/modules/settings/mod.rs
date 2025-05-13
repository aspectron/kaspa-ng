use crate::imports::*;

pub struct Settings {
    #[allow(dead_code)]
    runtime: Runtime,
    settings : crate::settings::Settings,
    wrpc_borsh_network_interface : NetworkInterfaceEditor,
    wrpc_json_network_interface : NetworkInterfaceEditor,
    grpc_network_interface : NetworkInterfaceEditor,
    reset_settings : bool,
}

impl Settings {
    pub fn new(runtime: Runtime) -> Self {
        Self { 
            runtime,
            settings : crate::settings::Settings::default(),
            wrpc_borsh_network_interface : NetworkInterfaceEditor::default(),
            wrpc_json_network_interface : NetworkInterfaceEditor::default(),
            grpc_network_interface : NetworkInterfaceEditor::default(),
            reset_settings : false,
        }
    }

    pub fn load(&mut self, settings : crate::settings::Settings) {
        self.settings = settings;

        self.wrpc_borsh_network_interface = NetworkInterfaceEditor::from(&self.settings.node.wrpc_borsh_network_interface);
        self.wrpc_json_network_interface = NetworkInterfaceEditor::from(&self.settings.node.wrpc_json_network_interface);
        self.grpc_network_interface = NetworkInterfaceEditor::from(&self.settings.node.grpc_network_interface);
    }

    pub fn change_current_network(&mut self, network : Network) {
        self.settings.node.network = network;
    }

    pub fn render_remote_settings(_core: &mut Core, ui: &mut Ui, settings : &mut NodeSettings) -> Option<&'static str> {

        let mut node_settings_error = None;

        CollapsingHeader::new(i18n("Remote p2p Node Configuration"))
        .default_open(true)
        .show(ui, |ui| {


            ui.horizontal_wrapped(|ui|{
                ui.label(i18n("Remote Connection:"));
                NodeConnectionConfigKind::iter().for_each(|kind| {
                    ui.radio_value(&mut settings.connection_config_kind, *kind, kind.to_string());
                });
            });

            match settings.connection_config_kind {
                NodeConnectionConfigKind::Custom => {

                    CollapsingHeader::new(i18n("wRPC Connection Settings"))
                        .default_open(true)
                        .show(ui, |ui| {


                            ui.horizontal(|ui|{
                                ui.label(i18n("wRPC Encoding:"));
                                WrpcEncoding::iter().for_each(|encoding| {
                                    ui.radio_value(&mut settings.wrpc_encoding, *encoding, encoding.to_string());
                                });
                            });


                            ui.horizontal(|ui|{
                                ui.label(i18n("wRPC URL:"));
                                ui.add(TextEdit::singleline(&mut settings.wrpc_url));
                                
                            });

                            if let Err(err) = KaspaRpcClient::parse_url(settings.wrpc_url.clone(), settings.wrpc_encoding, settings.network.into()) {
                                ui.label(
                                    RichText::new(err.to_string())
                                        .color(theme_color().warning_color),
                                );
                                node_settings_error = Some(i18n("Invalid wRPC URL"));
                            }
                        });
                    // cfg_if! {
                    //     if #[cfg(not(target_arch = "wasm32"))] {
                    //         ui.horizontal_wrapped(|ui|{
                    //             ui.label(i18n("Recommended arguments for the remote node: "));
                    //             ui.label(RichText::new("kaspad --utxoindex --rpclisten-borsh=0.0.0.0").code().font(FontId::monospace(14.0)).color(theme_color().strong_color));
                    //         });
                    //         ui.horizontal_wrapped(|ui|{
                    //             ui.label(i18n("If you are running locally, use: "));
                    //             ui.label(RichText::new("--rpclisten-borsh=127.0.0.1.").code().font(FontId::monospace(14.0)).color(theme_color().strong_color));
                    //         });
                    //     }
                    // }

                },
                NodeConnectionConfigKind::PublicServerCustom => {
                },
                NodeConnectionConfigKind::PublicServerRandom => {
                    ui.label(i18n("A random node will be selected on startup"));
                },
            }

        });

        node_settings_error
    }
}

impl ModuleT for Settings {

    fn init(&mut self, wallet : &mut Core) {
        self.load(wallet.settings.clone());
    }

    fn style(&self) -> ModuleStyle {
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

    fn deactivate(&mut self, _core: &mut Core) {
        #[cfg(not(target_arch = "wasm32"))]
        _core.storage.clear_settings();
    }

}

impl Settings {

    fn render_node_settings(
        &mut self,
        core: &mut Core,
        ui: &mut egui::Ui,
    ) {
        #[allow(unused_variables)]
        let half_width = ui.ctx().screen_rect().width() * 0.5;

        let mut node_settings_error = None;

        CollapsingHeader::new(i18n("Kaspa p2p Network & Node Connection"))
            .default_open(true)
            .show(ui, |ui| {


                CollapsingHeader::new(i18n("Kaspa Network"))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui|{
                            Network::iter().for_each(|network| {
                                ui.radio_value(&mut self.settings.node.network, *network, network.name());
                            });
                        });
                    });


                CollapsingHeader::new(i18n("Kaspa Node"))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui|{
                            KaspadNodeKind::iter().for_each(|node_kind| {
                                #[cfg(not(target_arch = "wasm32"))] {
                                    if !core.settings.developer.experimental_features_enabled() && matches!(*node_kind,KaspadNodeKind::IntegratedInProc|KaspadNodeKind::ExternalAsDaemon) {
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

                                ui.horizontal(|ui|{
                                    ui.label(i18n("Rusty Kaspa Daemon Path:"));
                                    ui.add(TextEdit::singleline(&mut self.settings.node.kaspad_daemon_binary));
                                });

                                let path = std::path::PathBuf::from(&self.settings.node.kaspad_daemon_binary);
                                if path.exists() && !path.is_file() {
                                    ui.label(
                                        RichText::new(format!("Rusty Kaspa Daemon not found at '{path}'", path = self.settings.node.kaspad_daemon_binary))
                                            .color(theme_color().error_color),
                                    );
                                    node_settings_error = Some("Rusty Kaspa Daemon not found");
                                }
                            },
                            _ => { }
                        }

                        #[cfg(not(target_arch = "wasm32"))]
                        if self.settings.node.node_kind.is_config_capable() {

                            CollapsingHeader::new(i18n("Cache Memory Size"))
                                .default_open(true)
                                .show(ui, |ui| {
                                    ui.horizontal_wrapped(|ui|{
                                        NodeMemoryScale::iter().for_each(|kind| {
                                            ui.radio_value(&mut self.settings.node.memory_scale, *kind, kind.to_string());
                                        });
                                    });
                                    ui.label(self.settings.node.memory_scale.describe());
                                });
                        }

                        #[cfg(not(target_arch = "wasm32"))]
                        if self.settings.node.node_kind.is_config_capable() {
                            CollapsingHeader::new(i18n("Data Storage"))
                                .default_open(true)
                                .show(ui, |ui| {
                                    ui.checkbox(&mut self.settings.node.kaspad_daemon_storage_folder_enable, i18n("Custom data storage folder"));
                                    if self.settings.node.kaspad_daemon_args.contains("--appdir") && self.settings.node.kaspad_daemon_storage_folder_enable {
                                        ui.colored_label(theme_color().warning_color, i18n("Your daemon arguments contain '--appdir' directive, which overrides the data storage folder setting."));
                                        ui.colored_label(theme_color().warning_color, i18n("Please remove the --appdir directive to continue."));
                                    } else if self.settings.node.kaspad_daemon_storage_folder_enable {
                                        ui.horizontal(|ui|{
                                            ui.label(i18n("Data Storage Folder:"));
                                            ui.add(TextEdit::singleline(&mut self.settings.node.kaspad_daemon_storage_folder));
                                        });

                                        let appdir = self.settings.node.kaspad_daemon_storage_folder.trim();
                                        if appdir.is_empty() {
                                            ui.colored_label(theme_color().error_color, i18n("Data storage folder must not be empty"));
                                        } else if !Path::new(appdir).exists() {
                                            ui.colored_label(theme_color().error_color, i18n("Data storage folder not found at"));
                                            ui.label(format!("\"{}\"",self.settings.node.kaspad_daemon_storage_folder.trim()));

                                            ui.add_space(4.);
                                            if ui.medium_button(i18n("Create Data Folder")).clicked() {
                                                if let Err(err) = std::fs::create_dir_all(appdir) {
                                                    runtime().error(format!("Unable to create data storage folder `{appdir}`: {err}"));
                                                }
                                            }
                                            ui.add_space(4.);

                                            node_settings_error = Some(i18n("Data storage folder not found"));
                                        }
                                    }
                                });
                        }

                        #[cfg(not(target_arch = "wasm32"))]
                        if core.settings.developer.custom_daemon_args_enabled() && self.settings.node.node_kind.is_config_capable() {
                            use kaspad_lib::args::Args;
                            use clap::error::ErrorKind as ClapErrorKind;
                            use crate::runtime::services::kaspa::Config;

                            ui.horizontal(|ui| {
                                ui.add_space(2.);
                                ui.checkbox(&mut self.settings.node.kaspad_daemon_args_enable, i18n("Activate custom daemon arguments"));
                            });

                            if self.settings.node.kaspad_daemon_args_enable {
                                ui.indent("kaspad_daemon_args", |ui| {
                                    ui.vertical(|ui| {
                                        ui.label(i18n("Resulting daemon arguments:"));
                                        ui.add_space(4.);

                                        let config = Config::from(self.settings.node.clone());
                                        let config = Vec::<String>::from(config).join(" ");
                                        ui.label(RichText::new(config).code().font(FontId::monospace(14.0)).color(theme_color().strong_color));
                                        ui.add_space(4.);


                                        ui.label(i18n("Custom arguments:"));
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
                                            node_settings_error = Some(i18n("Invalid daemon arguments"));
                                        }
                                    }
                                });
                            }
                        }

                    });

                if !self.grpc_network_interface.is_valid() {
                    node_settings_error = Some(i18n("Invalid gRPC network interface configuration"));
                } else {
                    self.settings.node.grpc_network_interface = self.grpc_network_interface.as_ref().try_into().unwrap(); //NetworkInterfaceConfig::try_from(&self.grpc_network_interface).unwrap();
                }

                if self.settings.node.node_kind == KaspadNodeKind::Remote {
                    node_settings_error = Self::render_remote_settings(core, ui, &mut self.settings.node);
                }

                #[cfg(not(target_arch = "wasm32"))]
                if self.settings.node.node_kind.is_config_capable() {

                    CollapsingHeader::new(i18n("Local p2p Node Configuration"))
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.vertical(|ui|{
                                CollapsingHeader::new(i18n("Client RPC"))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.vertical(|ui|{

                                            ui.checkbox(&mut self.settings.node.enable_wrpc_borsh, i18n("Public wRPC (Borsh)"));

                                            // ui.checkbox(&mut self.settings.node.enable_wrpc_json, i18n("Enable wRPC JSON"));
                                            // if self.settings.node.enable_wrpc_json {
                                            //     CollapsingHeader::new(i18n("wRPC JSON Network Interface & Port"))
                                            //         .default_open(true)
                                            //         .show(ui, |ui| {
                                            //             self.wrpc_json_network_interface.ui(ui);
                                            //         });
                                            // }

                                            ui.checkbox(&mut self.settings.node.enable_grpc, i18n("Enable gRPC"));
                                            if self.settings.node.enable_grpc {
                                                CollapsingHeader::new(i18n("gRPC Network Interface & Port"))
                                                    .default_open(true)
                                                    .show(ui, |ui| {
                                                        self.grpc_network_interface.ui(ui);
                                                    });
                                            }
                                        });

                                });
                            // });
                            
                                CollapsingHeader::new(i18n("p2p RPC"))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.vertical(|ui|{
                                            ui.checkbox(&mut self.settings.node.enable_upnp, i18n("Enable UPnP"));
                                        });
                                    });
                                });
                        });
                } // is_config_capable

            }); // Kaspa p2p Network & Node Connection

            if let Some(error) = node_settings_error {
                ui.add_space(4.);
                ui.label(
                    RichText::new(error)
                        .color(theme_color().error_color),
                );
                ui.add_space(4.);
                ui.label(i18n("Unable to change node settings until the problem is resolved"));

                ui.add_space(8.);

                if let Some(response) = ui.confirm_medium_cancel(Align::Max) {
                    if matches!(response, Confirm::Nack) {
                        self.settings.node = core.settings.node.clone();
                        self.grpc_network_interface = NetworkInterfaceEditor::from(&self.settings.node.grpc_network_interface);
                    }
                }

                ui.separator();

            } else if node_settings_error.is_none() {
                if let Some(restart) = self.settings.node.compare(&core.settings.node) {

                    ui.add_space(16.);
                    if let Some(response) = ui.confirm_medium_apply_cancel(Align::Max) {
                        match response {
                            Confirm::Ack => {

                                core.settings = self.settings.clone();
                                core.settings.store_sync().unwrap();

                                cfg_if! {
                                    if #[cfg(not(target_arch = "wasm32"))] {
                                        let storage_root = core.settings.node.kaspad_daemon_storage_folder_enable.then_some(core.settings.node.kaspad_daemon_storage_folder.as_str());
                                        core.storage.track_storage_root(storage_root);
                                    }
                                }

                                if restart {
                                    self.runtime.kaspa_service().update_services(&self.settings.node, None);
                                }
                            },
                            Confirm::Nack => {
                                self.settings = core.settings.clone();
                                self.grpc_network_interface = NetworkInterfaceEditor::from(&self.settings.node.grpc_network_interface);
                            }
                        }
                    }
                    ui.separator();
                }
            }
    }




    fn render_ui_settings(
        &mut self,
        core: &mut Core,
        ui: &mut egui::Ui,
    ) {


        CollapsingHeader::new(i18n("User Interface"))
            .default_open(false)
            .show(ui, |ui| {

                CollapsingHeader::new(i18n("Theme Color"))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                let theme_color = theme_color();
                                let current_theme_color_name = theme_color.name();
                                ui.menu_button(
                                    format!("{} ⏷", current_theme_color_name),
                                    |ui| {
                                        theme_colors().keys().for_each(|name| {
                                            if name.as_str() != current_theme_color_name
                                                && ui.button(name).clicked()
                                            {
                                                apply_theme_color_by_name(
                                                    ui.ctx(),
                                                    name,
                                                );
                                                core
                                                    .settings
                                                    .user_interface
                                                    .theme_color = name.to_string();
                                                core.store_settings();
                                                ui.close_menu();
                                            }
                                        });
                                    },
                                );
                            });
                        });

                        ui.add_space(1.);
                    });

                    CollapsingHeader::new(i18n("Theme Style"))
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let theme_style = theme_style();
                                let current_theme_style_name = theme_style.name();
                                ui.menu_button(
                                    format!("{} ⏷", current_theme_style_name),
                                    |ui| {
                                        theme_styles().keys().for_each(|name| {
                                            if name.as_str() != current_theme_style_name
                                                && ui.button(name).clicked()
                                            {
                                                apply_theme_style_by_name(ui.ctx(), name);
                                                core
                                                    .settings
                                                    .user_interface
                                                    .theme_style = name.to_string();
                                                core.store_settings();
                                                ui.close_menu();
                                            }
                                        });
                                    },
                                );
                            });
                            ui.add_space(1.);
                        });

                        if workflow_core::runtime::is_native() {
                            CollapsingHeader::new(i18n("Zoom"))
                                .default_open(true)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let zoom_factor = ui.ctx().zoom_factor();
                                        if ui
                                            .add_sized(
                                                Vec2::splat(24.),
                                                Button::new(RichText::new("-").size(18.)),
                                            )
                                            .clicked()
                                        {
                                            ui.ctx().set_zoom_factor(zoom_factor - 0.1);
                                        }
                                        ui.label(format!("{:.0}%", zoom_factor * 100.0));
                                        if ui
                                            .add_sized(
                                                Vec2::splat(24.),
                                                Button::new(RichText::new("+").size(18.)),
                                            )
                                            .clicked()
                                        {
                                            ui.ctx().set_zoom_factor(zoom_factor + 0.1);
                                        }
                                    });

                                    ui.add_space(1.);
                                });
                        }

                        if workflow_core::runtime::is_native() {

                            CollapsingHeader::new(i18n("Options"))
                                .default_open(true)
                                .show(ui, |ui| {

                                    ui.checkbox(&mut self.settings.user_interface.disable_frame, i18n("Disable Window Frame"));
                                    if self.settings.user_interface.disable_frame != core.settings.user_interface.disable_frame {
                                        ui.vertical(|ui| {
                                            ui.add_space(4.);
                                            ui.label(RichText::new(i18n("Application must be restarted for this setting to take effect.")).color(theme_color().warning_color));
                                            ui.label(RichText::new(i18n("Please select 'Apply' and restart the application.")).color(theme_color().warning_color));
                                            ui.add_space(4.);
                                        });
                                    }

                                    ui.add_space(1.);
                                });

                            if self.settings.user_interface.disable_frame != core.settings.user_interface.disable_frame {
                                ui.add_space(16.);
                                if let Some(response) = ui.confirm_medium_apply_cancel(Align::Max) {
                                    match response {
                                        Confirm::Ack => {
                                            core.settings.user_interface.disable_frame = self.settings.user_interface.disable_frame;
                                            core.settings.store_sync().unwrap();
                                        },
                                        Confirm::Nack => {
                                            self.settings.user_interface.disable_frame = core.settings.user_interface.disable_frame;
                                        }
                                    }
                                }
                                ui.separator();
                            }
                        }
            });

    }

    fn render_settings(
        &mut self,
        core: &mut Core,
        ui: &mut egui::Ui,
    ) {

        self.render_node_settings(core,ui);

        self.render_ui_settings(core,ui);

        CollapsingHeader::new(i18n("Services"))
            .default_open(true)
            .show(ui, |ui| {

                CollapsingHeader::new(i18n("Market Monitor"))
                    .default_open(true)
                    .show(ui, |ui| {
                        if ui.checkbox(&mut self.settings.market_monitor, i18n("Enable Market Monitor")).changed() {
                            core.settings.market_monitor = self.settings.market_monitor;
                            self.runtime.market_monitor_service().enable(core.settings.market_monitor);
                            core.store_settings();
                        }
                    });

                #[cfg(not(target_arch = "wasm32"))]
                CollapsingHeader::new(i18n("Check for Updates"))
                    .default_open(true)
                    .show(ui, |ui| {
                        if ui.checkbox(&mut self.settings.update_monitor, i18n("Check for Software Updates on GitHub")).changed() {
                            core.settings.update_monitor = self.settings.update_monitor;
                            self.runtime.update_monitor_service().enable(core.settings.update_monitor);
                            core.store_settings();
                        }
                    });    
            });

        CollapsingHeader::new(i18n("Network Fee Estimator"))
            .default_open(false)
            .show(ui, |ui| {
                ui.vertical(|ui|{
                    EstimatorMode::iter().for_each(|kind| {
                        ui.radio_value(&mut self.settings.estimator.mode, *kind, i18n(kind.describe()));
                    });
                    
                    if self.settings.estimator.mode != core.settings.estimator.mode {
                        core.settings.estimator.mode = self.settings.estimator.mode;
                        core.store_settings();
                    }
                });
            });
            
        #[cfg(not(target_arch = "wasm32"))]
        core.storage.clone().render_settings(core, ui);

        CollapsingHeader::new(i18n("Advanced"))
            .default_open(false)
            .show(ui, |ui| {

                ui.horizontal(|ui| {
                    ui.add_space(2.);
                    ui.vertical(|ui|{
                        ui.checkbox(&mut self.settings.developer.enable, i18n("Developer Mode"));
                        if !self.settings.developer.enable {
                            ui.label(i18n("Developer mode enables advanced and experimental features"));
                        }
                    });
                });

                if self.settings.developer.enable {
                    ui.indent("developer_mode_settings", |ui | {

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
                            i18n("Enable custom daemon arguments")
                        ).on_hover_text_at_pointer(
                            i18n("Allow custom arguments for the Rusty Kaspa daemon")
                        );
                        
                        ui.checkbox(
                            &mut self.settings.developer.disable_password_restrictions, 
                            i18n("Disable password safety rules")
                        ).on_hover_text_at_pointer(
                            i18n("Removes security restrictions, allows for single-letter passwords")
                        );
                        
                        ui.checkbox(
                            &mut self.settings.developer.market_monitor_on_testnet, 
                            i18n("Show balances in alternate currencies for testnet coins")
                        ).on_hover_text_at_pointer(
                            i18n("Shows balances in alternate currencies (BTC, USD) when using testnet coins as if you are on mainnet")
                        );

                        #[cfg(not(target_arch = "wasm32"))]
                        ui.checkbox(
                            &mut self.settings.developer.enable_screen_capture, 
                            i18n("Enable screen capture")
                        ).on_hover_text_at_pointer(
                            i18n("Allows you to take screenshots from within the application")
                        );
                    });
                }

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
                        if ui.medium_button(i18n("Reset Settings")).clicked() {
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
    }
}

