use crate::imports::*;

pub struct Menu<'core> {
    core: &'core mut Core,
}

impl<'core> Menu<'core> {
    pub fn new(core: &'core mut Core) -> Self {
        Self { core }
    }

    fn select<T>(&mut self)
    where
        T: ModuleT + 'static,
    {
        self.core.select::<T>();
    }

    pub fn render(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            ui.columns(2, |cols| {
                cols[0].horizontal(|ui| {
                    ui.menu_button("File", |ui| {
                        #[cfg(not(target_arch = "wasm32"))]
                        if ui.button("Quit").clicked() {
                            ui.ctx().send_viewport_cmd(ViewportCommand::Close)
                        }
                        ui.separator();
                        ui.label(" ~ Debug Modules ~");
                        ui.label(" ");

                        let (tests, mut modules): (Vec<_>, Vec<_>) = self
                            .core
                            .modules()
                            .values()
                            .cloned()
                            .partition(|module| module.name().starts_with('~'));

                        tests.into_iter().for_each(|module| {
                            if ui.button(module.name()).clicked() {
                                self.core.select_with_type_id(module.type_id());
                                ui.close_menu();
                            }
                        });

                        ui.label(" ");

                        modules.sort_by(|a, b| a.name().partial_cmp(b.name()).unwrap());
                        modules.into_iter().for_each(|module| {
                            if ui.button(module.name()).clicked() {
                                self.core.select_with_type_id(module.type_id());
                                ui.close_menu();
                            }
                        });
                    });

                    ui.separator();
                    if ui.button("Overview").clicked() {
                        self.select::<modules::Overview>();
                    }
                    ui.separator();
                    if ui.button("Wallet").clicked() {
                        if self.core.state().is_open() {
                            self.select::<modules::AccountManager>();
                        } else {
                            self.select::<modules::WalletOpen>();
                        }
                    }
                    ui.separator();

                    if ui.button("Settings").clicked() {
                        self.select::<modules::Settings>();
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.separator();
                        if ui.button("Node").clicked() {
                            self.select::<modules::Node>();
                        }
                    }

                    ui.separator();
                    if ui.button("Metrics").clicked() {
                        self.select::<modules::Metrics>();
                    }

                    ui.separator();
                    if ui.button("Block DAG").clicked() {
                        self.select::<modules::BlockDag>();
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.separator();
                        if ui.button("Logs").clicked() {
                            self.select::<modules::Logs>();
                        }
                    }

                    ui.separator();
                });

                cols[1].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let dictionary = i18n::dictionary();
                    // use egui_phosphor::light::TRANSLATE;
                    #[allow(clippy::useless_format)]
                    ui.menu_button(format!("{} ⏷", dictionary.current_title()), |ui| {
                        // ui.menu_button(RichText::new(format!("{TRANSLATE} ⏷")).size(18.), |ui| {
                        dictionary
                            .enabled_languages()
                            .into_iter()
                            .for_each(|(code, lang)| {
                                if ui.button(lang).clicked() {
                                    self.core.settings.language_code = code.to_string();
                                    dictionary
                                        .activate_language_code(code)
                                        .expect("Unable to activate language");
                                    ui.close_menu();
                                }
                            });
                    });

                    ui.separator();

                    PopupPanel::new(
                        ui,
                        "display_settings",
                        egui_phosphor::light::MONITOR,
                        |ui| {
                            ui.label("hello world");

                            if ui.button("Change").clicked() {
                                ui.ctx().set_visuals(Visuals::light());
                            }

                            if self.core.settings.developer.enable_screen_capture() {
                                ui.add_space(8.);
                                use egui_phosphor::thin::CAMERA;
                                if ui.button(CAMERA).clicked() {
                                    ui.ctx()
                                        .send_viewport_cmd(egui::ViewportCommand::Screenshot);
                                }
                            }
                        },
                    )
                    .with_min_width(64.)
                    .build(ui);

                    // // let icon_size = theme.panel_icon_size();
                    // let icon = CompositeIcon::new(egui_phosphor::light::MONITOR).icon_size(18.);
                    // // .padding(Some(icon_padding));
                    // // if ui.add_enabled(true, icon).clicked() {
                    // if ui.add(icon).clicked() {
                    //     // close(self.this);
                    // }

                    // if ui.button("Theme").clicked() {
                    //     self.select::<modules::Logs>();
                    // }
                    ui.separator();
                });
            });
        });
    }
}
