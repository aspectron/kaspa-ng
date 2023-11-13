use crate::imports::*;

pub struct Welcome {
    #[allow(dead_code)]
    interop: Interop,
    settings : Settings,
}

impl Welcome {
    pub fn new(interop: Interop) -> Self {
        Self { 
            interop, 
            settings : Settings::default(),
        }
    }
}

impl ModuleT for Welcome {

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

        ui.heading("Welcome to Kaspa NG");
        ui.add_space(16.0);
        ui.label("Please configure your Kaspa NG settings");
        ui.add_space(16.0);

        CollapsingHeader::new("Settings")
            .default_open(true)
            .show(ui, |ui| {
                CollapsingHeader::new("Kaspa Network")
                    .default_open(true)
                    .show(ui, |ui| {

                            ui.horizontal_wrapped(|ui| {
                                Network::iter().for_each(|network| {
                                    ui.radio_value(&mut self.settings.node.network, *network, network.describe());
                                });
                            });
                    });
                
                CollapsingHeader::new("Kaspa p2p Node & Connection")
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
                            ].iter().for_each(|node| {
                                ui.radio_value(&mut self.settings.node.node_kind, *node, node.describe());
                            });
                        });
                    });

                CollapsingHeader::new("User Interface")
                    .default_open(true)
                    .show(ui, |ui| {

                        ui.horizontal(|ui| {

                            ui.label("Language:");

                            // let dict = i18n::dict();
                            let language_code = core.settings.language_code.clone();
                            let dictionary = i18n::dictionary();
                            let language = dictionary.language_title(language_code.as_str()).unwrap();//.unwrap();
                            egui::ComboBox::from_id_source("language_selector")
                                .selected_text(language)
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap = Some(false);
                                    ui.set_min_width(60.0);
                                    dictionary.enabled_languages().into_iter().for_each(|(code,lang)| {
                                        ui.selectable_value(&mut self.settings.language_code, code.to_string(), lang);
                                    });
                                });

                            ui.add_space(16.);
                            ui.label("Theme:");

                            egui::ComboBox::from_id_source("theme_selector")
                                .selected_text("Dark")
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap = Some(false);
                                    ui.set_min_width(60.0);
                                    ["Dark","Light"].into_iter().for_each(|theme| {
                                        ui.selectable_value(&mut self.settings.theme, theme.to_string(), theme);
                                    });
                                });
                        });        
                    });

                ui.add_space(32.0);
                ui.horizontal(|ui| {
                    ui.add_space(
                        ui.available_width()
                            - 16.
                            - (theme().medium_button_size.x + ui.spacing().item_spacing.x),
                    );
                    if ui.medium_button(format!("{} {}", egui_phosphor::light::CHECK, "Apply")).clicked() {
                        let mut settings = self.settings.clone();
                        settings.initialized = true;
                        settings.version.clear(); // triggers changelog
                        settings.store_sync().expect("Unable to store settings");
                        self.interop.kaspa_service().update_services(&self.settings.node);
                        core.settings = settings.clone();
                        core.get_mut::<modules::Settings>().load(settings);
                        core.select::<modules::Overview>();
                    }
                });
                ui.separator();
        });
        
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            ui.label(format!("Kaspa NG v{}  •  Rusty Kaspa v{}", env!("CARGO_PKG_VERSION"), kaspa_wallet_core::version()));
            ui.hyperlink_to(
                "https://kaspa.org",
                "https://kaspa.org",
            );
    
        });
    }
}
