use egui_phosphor::thin::TRANSLATE;
use workflow_core::runtime;

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
                    if self.core.settings.developer.enable && self.core.debug {
                        self.render_debug(ui);
                        ui.separator();
                    }

                    if self.core.device().single_pane() {
                        ui.menu_button("Kaspa NG", |ui| {
                            self.render_menu(ui);
                        });
                    } else {
                        self.render_menu(ui);
                        ui.separator();
                    }
                });

                cols[1].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let dictionary = i18n::dictionary();
                    let lang_menu = if self.core.device().orientation() == Orientation::Portrait {
                        RichText::new(TRANSLATE).size(18.)
                    } else {
                        RichText::new(format!("{} ⏷", dictionary.current_title()))
                    };
                    #[allow(clippy::useless_format)]
                    ui.menu_button(lang_menu, |ui| {
                        dictionary
                            .enabled_languages()
                            .into_iter()
                            .for_each(|(code, lang)| {
                                let line_height = match code {
                                    "ar" | "fa" => Some(26.),
                                    "zh" | "ko" | "ja" => Some(20.),
                                    "hi" | "he" => Some(10.),
                                    _ => None,
                                };

                                let size = vec2(100., 24.);
                                if ui
                                    .add_sized(
                                        size,
                                        Button::new(RichText::new(lang).line_height(line_height)),
                                    )
                                    .clicked()
                                {
                                    self.core.settings.language_code = code.to_string();
                                    dictionary
                                        .activate_language_code(code)
                                        .expect("Unable to activate language");
                                    self.core.settings.language_code = code.to_string();
                                    self.core.store_settings();
                                    ui.close_menu();
                                }
                            });
                    });

                    ui.separator();

                    PopupPanel::new(
                        PopupPanel::id(ui, "display_settings"),
                        |ui| {
                            ui.add(
                                Label::new(RichText::new(egui_phosphor::light::MONITOR).size(16.))
                                    .sense(Sense::click()),
                            )
                        },
                        |ui, _close_popup| {
                            Grid::new("display_popup_grid")
                                .num_columns(2)
                                .spacing([4.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label(i18n("Theme Color"));
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            let current_theme_color_name = theme_color().name();
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
                                                            self.core
                                                                .settings
                                                                .user_interface
                                                                .theme_color = name.to_string();
                                                            self.core.store_settings();
                                                            ui.close_menu();
                                                        }
                                                    });
                                                },
                                            );
                                        });
                                        ui.add_space(1.);
                                    });
                                    ui.end_row();

                                    ui.label(i18n("Theme Style"));
                                    ui.horizontal(|ui| {
                                        let current_theme_style_name = theme_style().name();
                                        ui.menu_button(
                                            format!("{} ⏷", current_theme_style_name),
                                            |ui| {
                                                theme_styles().keys().for_each(|name| {
                                                    if name.as_str() != current_theme_style_name
                                                        && ui.button(name).clicked()
                                                    {
                                                        apply_theme_style_by_name(ui.ctx(), name);
                                                        self.core
                                                            .settings
                                                            .user_interface
                                                            .theme_style = name.to_string();
                                                        self.core.store_settings();
                                                        ui.close_menu();
                                                    }
                                                });
                                            },
                                        );
                                    });
                                    ui.end_row();

                                    if runtime::is_native() || runtime::is_chrome_extension() {
                                        ui.label(i18n("Zoom"));
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
                                        ui.end_row();
                                    }
                                });

                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                if self.core.settings.developer.screen_capture_enabled() {
                                    ui.sized_separator(vec2(60., 8.));
                                    ui.vertical_centered(|ui| {
                                        use egui_phosphor::light::CAMERA;
                                        let mut response = ui.add_sized(
                                            vec2(32., 32.),
                                            Button::new(RichText::new(CAMERA).size(20.)),
                                        );

                                        response =
                                            response.on_hover_text(i18n("Capture a screenshot"));

                                        if response.clicked() {
                                            *_close_popup = true;
                                            ui.ctx().send_viewport_cmd(
                                                egui::ViewportCommand::Screenshot,
                                            );
                                        }
                                    });
                                }
                            }
                        },
                    )
                    .with_min_width(64.)
                    .build(ui);

                    if self.core.notifications().has_some() {
                        ui.separator();
                        self.core.notifications().render(ui);
                    }
                });
            });
        });
    }

    pub fn render_menu(&mut self, ui: &mut Ui) {
        if ui.button(i18n("Overview")).clicked() {
            self.select::<modules::Overview>();
            ui.close_menu();
        }
        ui.separator();

        #[allow(clippy::collapsible_else_if)]
        if self.core.state().is_open() {
            if ui.button(i18n("Wallet")).clicked() {
                self.select::<modules::AccountManager>();
                ui.close_menu();
            }
        } else {
            if ui.button(i18n("Wallet")).clicked() {
                self.select::<modules::WalletOpen>();
                ui.close_menu();
            }
        }
        // if ui.button(i18n("Wallet")).clicked() {
        //     if self.core.state().is_open() {
        //         self.select::<modules::AccountManager>();
        //     } else {
        //         self.select::<modules::WalletOpen>();
        //     }
        //     ui.close_menu();
        // }

        #[cfg(not(feature = "lean"))]
        {
            ui.separator();
            if ui.button(i18n("Metrics")).clicked() {
                self.select::<modules::Metrics>();
                ui.close_menu();
            }

            ui.separator();
            if ui.button(i18n("Block DAG")).clicked() {
                self.select::<modules::BlockDag>();
                ui.close_menu();
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if self.core.settings.node.node_kind.is_local() {
                ui.separator();
                if ui.button(i18n("Node")).clicked() {
                    self.select::<modules::Node>();
                    ui.close_menu();
                }
            }
        }

        ui.separator();

        if ui.button(i18n("Settings")).clicked() {
            self.select::<modules::Settings>();
            ui.close_menu();
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if self.core.settings.node.node_kind.is_local() {
                ui.separator();
                if ui.button(i18n("Logs")).clicked() {
                    self.select::<modules::Logs>();
                    ui.close_menu();
                }
            }
        }
    }

    pub fn render_debug(&mut self, ui: &mut Ui) {
        ui.menu_button("Debug", |ui| {
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button(i18n("Quit")).clicked() {
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
    }
}
