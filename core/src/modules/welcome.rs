use crate::imports::*;

pub struct Welcome {
    #[allow(dead_code)]
    runtime: Runtime,
    settings : Settings,
}

impl Welcome {
    pub fn new(runtime: Runtime) -> Self {

        #[allow(unused_mut)]
        let mut settings = Settings::default();

        #[cfg(target_arch = "wasm32")] {
            settings.node.node_kind = KaspadNodeKind::Remote;
        }

        Self { 
            runtime, 
            settings,
        }
    }

    pub fn render_native(
        &mut self,
        core: &mut Core,
        ui: &mut egui::Ui,
    ) {

        let mut error = None;

        ui.heading(i18n("Welcome to Kaspa NG"));
        ui.add_space(16.0);
        ui.label(i18n("Please configure your Kaspa NG settings"));
        ui.add_space(16.0);

        CollapsingHeader::new(i18n("Settings"))
            .default_open(true)
            .show(ui, |ui| {
                CollapsingHeader::new(i18n("Kaspa Network"))
                    .default_open(true)
                    .show(ui, |ui| {

                            ui.horizontal_wrapped(|ui| {
                                Network::iter().for_each(|network| {
                                    ui.radio_value(&mut self.settings.node.network, *network, format!("{} ({})",network.name(),network.describe()));

                                });
                            });

                            match self.settings.node.network {
                                Network::Mainnet => {
                                    // ui.colored_label(theme_color().warning_color, i18n("Please note that this is a beta release. Until this message is removed, please avoid using the wallet with mainnet funds."));
                                }
                                Network::Testnet10 => { }
                            }
                        });
                
                CollapsingHeader::new(i18n("Kaspa p2p Node & Connection"))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            // KaspadNodeKind::iter().for_each(|node| {
                            [
                                KaspadNodeKind::Disable,
                                KaspadNodeKind::Remote,
                                #[cfg(not(target_arch = "wasm32"))]
                                KaspadNodeKind::IntegratedAsDaemon,
                                // KaspadNodeKind::ExternalAsDaemon,
                                // KaspadNodeKind::IntegratedInProc,
                            ].iter().for_each(|node_kind| {
                                ui.radio_value(&mut self.settings.node.node_kind, *node_kind, node_kind.to_string()).on_hover_text_at_pointer(node_kind.describe());
                            });
                        });

                        if self.settings.node.node_kind == KaspadNodeKind::Remote {
                            error = crate::modules::settings::Settings::render_remote_settings(core,ui,&mut self.settings.node);
                        }
                    });

                CollapsingHeader::new(i18n("User Interface"))
                    .default_open(true)
                    .show(ui, |ui| {

                        ui.horizontal(|ui| {

                            ui.label(i18n("Language:"));

                            let language_code = core.settings.language_code.clone();
                            let dictionary = i18n::dictionary();
                            let language = dictionary.language_title(language_code.as_str()).unwrap();//.unwrap();
                            egui::ComboBox::from_id_salt("language_selector")
                                .selected_text(language)
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                                    ui.set_min_width(60.0);
                                    dictionary.enabled_languages().into_iter().for_each(|(code,lang)| {
                                        ui.selectable_value(&mut self.settings.language_code, code.to_string(), lang);
                                    });
                                });

                            ui.add_space(16.);
                            ui.label(i18n("Theme Color:"));

                            let mut theme_color = self.settings.user_interface.theme_color.clone();
                            egui::ComboBox::from_id_salt("theme_color_selector")
                                .selected_text(theme_color.as_str())
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                                    ui.set_min_width(60.0);
                                    theme_colors().keys().for_each(|name| {
                                        ui.selectable_value(&mut theme_color, name.to_string(), name);
                                    });
                                });
                                
                            if theme_color != self.settings.user_interface.theme_color {
                                self.settings.user_interface.theme_color = theme_color;
                                apply_theme_color_by_name(ui.ctx(), self.settings.user_interface.theme_color.clone());
                            }

                            ui.add_space(16.);
                            ui.label(i18n("Theme Style:"));

                            let mut theme_style = self.settings.user_interface.theme_style.clone();
                            egui::ComboBox::from_id_salt("theme_style_selector")
                                .selected_text(theme_style.as_str())
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                                    ui.set_min_width(60.0);
                                    theme_styles().keys().for_each(|name| {
                                        ui.selectable_value(&mut theme_style, name.to_string(), name);
                                    });
                                });
                                
                            if theme_style != self.settings.user_interface.theme_style {
                                self.settings.user_interface.theme_style = theme_style;
                                apply_theme_style_by_name(ui.ctx(), self.settings.user_interface.theme_style.clone());
                            }
                        });        
                    });

                ui.add_space(32.0);
                if let Some(error) = error {
                    ui.vertical_centered(|ui| {
                        ui.colored_label(theme_color().alert_color, error);
                    });
                    ui.add_space(32.0);
                } else {
                    
                    ui.horizontal(|ui| {
                        ui.add_space(
                            ui.available_width()
                                - 16.
                                - (theme_style().medium_button_size.x + ui.spacing().item_spacing.x),
                        );
                        if ui.medium_button(format!("{} {}", egui_phosphor::light::CHECK, i18n("Apply"))).clicked() {
                            let mut settings = self.settings.clone();
                            settings.initialized = true;
                            let message = i18n("Unable to store settings");
                            settings.store_sync().expect(message);
                            self.runtime.kaspa_service().update_services(&self.settings.node, None);
                            core.settings = settings.clone();
                            core.get_mut::<modules::Settings>().load(settings);
                            cfg_if!{
                                if #[cfg(not(target_arch = "wasm32"))] {
                                    core.select::<modules::Changelog>();
                                } else {
                                    core.select::<modules::Overview>();
                                }
                            }
                        }
                    });
                }

                ui.separator();
        });
        
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            // ui.colored_label(theme_color().alert_color, "Please note - this is a beta release - Kaspa NG is still in early development and is not yet ready for production use.");
            // ui.add_space(32.0);
            ui.label(format!("Kaspa NG v{}  •  Rusty Kaspa v{}", env!("CARGO_PKG_VERSION"), kaspa_wallet_core::version()));
            ui.hyperlink_to(
                "https://kaspa.org",
                "https://kaspa.org",
            );
    
        });
    }

    pub fn render_web(
        &mut self,
        core: &mut Core,
        ui: &mut egui::Ui,
    ) {
        let mut proceed = false;

        Panel::new(self)
            .with_caption(i18n("Welcome to Kaspa NG"))
            .with_header(|_this, ui| {
                ui.label(i18n("Please select Kaspa network"));
            })
            .with_body(|this, ui| {
                Network::iter().for_each(|network| {
                    if ui.add_sized(
                            theme_style().large_button_size,
                            CompositeButton::opt_image_and_text(
                                None,
                                Some(network.name().into()),
                                Some(network.describe().into()),
                            ),
                        )
                        .clicked()
                    {
                        this.settings.node.network = *network;
                        proceed = true;
                    }

                    ui.add_space(8.);
                });

                // ui.add_space(32.0);
                // ui.colored_label(theme_color().alert_color, RichText::new("β").size(64.0));
                // ui.add_space(8.0);
                // ui.colored_label(theme_color().alert_color, "Please note - this is a beta release - Kaspa NG is still in early development and is not yet ready for production use.");
            })
            .render(ui);        

        if proceed {
            let mut settings = self.settings.clone();
            settings.initialized = true;
            let message = i18n("Unable to store settings");
            settings.store_sync().expect(message);
            core.settings = settings.clone();
            self.runtime.kaspa_service().update_services(&settings.node, None);

            core.get_mut::<modules::Settings>().load(settings);
            core.select::<modules::Overview>();
        }

    }

}

impl ModuleT for Welcome {

    fn style(&self) -> ModuleStyle {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                ModuleStyle::Mobile
            } else {
                ModuleStyle::Default
            }
        }
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                self.render_native(core, ui)
            } else {
                self.render_web(core, ui)
            }
        }
    }

}
