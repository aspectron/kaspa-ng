use egui::load::Bytes;
use egui_phosphor::thin::TRANSLATE;
use std::borrow::Cow;
use workflow_core::runtime;

use crate::{imports::*, modules::account_manager::AccountManagerSection};

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

    pub fn render_single_pane_menu_closed(&mut self, ui: &mut Ui, device: &Device) {
        // if ui.add(
        //     Image::new(ImageSource::Bytes { uri : Cow::Borrowed("bytes://wallet.svg"), bytes : Bytes::Static(include_bytes!("../resources/svg/dark/wallet.svg"))})
        //         .fit_to_exact_size(Vec2::splat(device.top_icon_size()))
        //         .maintain_aspect_ratio(true)
        //         .texture_options(TextureOptions::NEAREST),
        //         // .texture_options(TextureOptions::LINEAR),
        //     ).clicked() {

        //         self.core.select::<modules::WalletOpen>();
        //     }

        if ui
            .add(
                Label::new(
                    RichText::new(egui_phosphor::light::WALLET).size(device.top_icon_size()),
                )
                .sense(Sense::click()),
            )
            .clicked()
        {
            self.core.select::<modules::WalletOpen>();
        }
    }

    pub fn render_single_pane_menu_open(&mut self, ui: &mut Ui, device: &Device) {
        // if ui.add(
        //     Label::new(
        //         RichText::new(egui_phosphor::light::WALLET)
        //             .size(device.top_icon_size()),
        //     )
        //     .sense(Sense::click()),
        // ).clicked() {
        //     self.core.select::<modules::WalletOpen>();
        // }

        // if ui.add(
        //     Label::new(
        //         RichText::new(egui_phosphor::light::WALLET)
        //             .size(device.top_icon_size()),
        //     )
        //     .sense(Sense::click()),
        // ).clicked() {
        // self.core.select::<modules::WalletOpen>();

        // ui.image(source)
        // ui.add(
        //     Image::new(ImageSource::Bytes { uri : Cow::Borrowed("bytes://logo-transparent.svg"), bytes : Bytes::Static(crate::app::KASPA_NG_ICON_TRANSPARENT_SVG)})

        //     // egui::Image::new(egui::include_image!("../assets/ferris.png"))
        //         // .max_height(32.0)
        //         // .max_width(32.0)
        //         .fit_to_exact_size(Vec2::splat(26.0))
        //         // .max_size(Vec2::splat(32.0))
        //         // .load_for_size(ui.ctx(), vec2(32.0, 32.0)).unwrap()
        //         .maintain_aspect_ratio(true)
        //         .texture_options(TextureOptions::LINEAR),
        //         // .corner_radius(10.0),
        // );

        // ui.separator();

        // use modules::account_manager::menus::*;
        // use modules::account_manager::RenderContext;

        if self.core.module().type_id() != TypeId::of::<modules::AccountManager>() {
            // if ui.add(
            //     Image::new(ImageSource::Bytes { uri : Cow::Borrowed("bytes://wallet.svg"), bytes : Bytes::Static(include_bytes!("../resources/svg/dark/wallet.svg"))})
            //         .fit_to_exact_size(Vec2::splat(device.top_icon_size()))
            //         .maintain_aspect_ratio(true)
            //         // .texture_options(TextureOptions::NEAREST),
            //         .texture_options(TextureOptions::LINEAR),
            //     ).clicked() {

            //         self.core.select::<modules::AccountManager>();
            //     }

            if ui
                .add(
                    Label::new(
                        RichText::new(egui_phosphor::light::WALLET.to_string())
                            .size(device.top_icon_size()),
                    )
                    .sense(Sense::click()),
                )
                .clicked()
            {
                self.core.select::<modules::AccountManager>();
            }
        } else {
            // if ui.add(
            //     Image::new(ImageSource::Bytes { uri : Cow::Borrowed("bytes://wallet.svg"), bytes : Bytes::Static(include_bytes!("../resources/svg/dark/wallet.svg"))})
            //         .fit_to_exact_size(Vec2::splat(device.top_icon_size()))
            //         .maintain_aspect_ratio(true)
            //         // .texture_options(TextureOptions::NEAREST),
            //         .texture_options(TextureOptions::LINEAR),
            //     ).clicked() {

            //         self.core.get_mut::<modules::AccountManager>().change_section(AccountManagerSection::Transactions);

            //     }

            if ui
                .add(
                    Label::new(
                        RichText::new(egui_phosphor::light::WALLET.to_string())
                            .size(device.top_icon_size()),
                    )
                    .sense(Sense::click()),
                )
                .clicked()
            {
                self.core
                    .get_mut::<modules::AccountManager>()
                    .change_section(AccountManagerSection::Transactions);
            }

            // WalletMenu::default().render_selector(
            //     self.core,
            //     ui,
            //     device.screen_size.y * 0.8,
            //     |ui| {

            //         // let mut layout_job = LayoutJob::default();
            //         // let style = Style::default();
            //         let layout_job = layout_job(vec![
            //             RichText::new(egui_phosphor::light::WALLET).size(device.top_icon_size()), //.append_to(&mut layout_job, &style, FontSelection::default(), Align::Center);
            //             RichText::new(" ⏷"), //.append_to(&mut layout_job, &style, FontSelection::default(), Align::Center);
            //         ]);
            //         ui.add(
            //             Label::new(
            //                 // RichText::new(format!("{} ⏷", egui_phosphor::light::WALLET))
            //                 layout_job
            //                     ,
            //             )
            //             .sense(Sense::click()),
            //         )
            //     },
            // );

            // }

            // let network = self.core.network();
            // let current_daa_score = self.core.state().current_daa_score();
            // let mut account_manager = self
            //     .core
            //     .modules()
            //     .get(&TypeId::of::<modules::AccountManager>())
            //     .unwrap()
            //     .clone();
            // let mut account_manager = account_manager.get_mut::<modules::AccountManager>();
            // if let Some(account) = account_manager.account() {
            //     // let rc = if let Ok(rc) = RenderContext::new(account,self.core) {
            //     let rc = if let Ok(rc) =
            //         RenderContext::new(account, network.into(), current_daa_score)
            //     {
            //         rc
            //     } else {
            //         ui.label("RC");
            //         return;
            //     };
            //     //  {

            //     AccountMenu::default().render_selector(
            //         self.core,
            //         ui,
            //         device.screen_size.y * 0.8,
            //         &mut account_manager,
            //         &rc,
            //         |ui| {
            //             let layout_job = layout_job(vec![
            //                 RichText::new(egui_phosphor::light::COINS).size(device.top_icon_size()), //.append_to(&mut layout_job, &style, FontSelection::default(), Align::Center);
            //                 RichText::new(" ⏷"), //.append_to(&mut layout_job, &style, FontSelection::default(), Align::Center);
            //                 // RichText::new(egui_phosphor::light::COINS).size(device.top_icon_size()), //.append_to(&mut layout_job, &style, FontSelection::default(), Align::Center);
            //                 // RichText::new(" ⏷"), //.append_to(&mut layout_job, &style, FontSelection::default(), Align::Center);
            //             ]);

            //             ui.add(
            //                 Label::new(
            //                     layout_job
            //                     // RichText::new(format!("{} ⏷", egui_phosphor::light::COINS))
            //                     //     .size(device.top_icon_size()),
            //                 )
            //                 .sense(Sense::click()),
            //             )
            //         },
            //     );
            // }

            // if ui
            //     .add(
            //         Label::new(
            //             RichText::new(egui_phosphor::light::LIST_BULLETS)
            //                 .size(device.top_icon_size()),
            //         )
            //         .sense(Sense::click()),
            //     )
            //     .clicked()
            // {
            //     // self.core.select::<modules::WalletOpen>();
            //     // let mut account_manager = self.core.get_mut::<modules::AccountManager>();
            //     account_manager.change_section(AccountManagerSection::Transactions);
            // }
        }

        // if ui
        //     .add(
        //         Label::new(
        //             RichText::new(egui_phosphor::light::LIST_BULLETS).size(device.top_icon_size()),
        //         )
        //         .sense(Sense::click()),
        //     )
        //     .clicked()
        // {
        //     // self.core.select::<modules::WalletOpen>();
        // }

        // if ui
        //     .add(
        //         Label::new(RichText::new(egui_phosphor::light::COINS).size(device.top_icon_size()))
        //             .sense(Sense::click()),
        //     )
        //     .clicked()
        // {
        //     // self.core.select::<modules::WalletOpen>();
        // }
    }

    pub fn render_single_pane_menu(&mut self, ui: &mut Ui, device: &Device) {
        ui.add(
            Image::new(ImageSource::Bytes {
                uri: Cow::Borrowed("bytes://logo-transparent.svg"),
                bytes: Bytes::Static(crate::app::KASPA_NG_ICON_TRANSPARENT_SVG),
            })
            .fit_to_exact_size(Vec2::splat(device.top_icon_size()))
            .maintain_aspect_ratio(true)
            // .texture_options(TextureOptions::NEAREST),
            .texture_options(TextureOptions::LINEAR),
        );

        ui.separator();

        if self.core.state().is_open() {
            self.render_single_pane_menu_open(ui, device);
        } else {
            self.render_single_pane_menu_closed(ui, device);
        }
    }

    pub fn render(&mut self, ui: &mut Ui) {
        let device = self.core.device().clone();

        egui::menu::bar(ui, |ui| {
            ui.columns(2, |cols| {
                cols[0].horizontal(|ui| {
                    if device.single_pane() {
                        self.render_single_pane_menu(ui, &device);
                    } else {
                        if self.core.settings.developer.enable && self.core.debug {
                            self.render_debug(ui);
                            ui.separator();
                        }

                        if self.core.device().single_pane() {
                            ui.menu_button("Kaspa NG", |ui| {
                                self.render_desktop_menu(ui);
                            });
                        } else {
                            self.render_desktop_menu(ui);
                            ui.separator();
                        }
                    }
                });

                cols[1].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if device.single_pane() {
                        self.render_single_pane_menu_rtl(ui, &device);
                    } else {
                        self.render_desktop_menu_rtl(ui, &device);
                    }
                });
            });
        });
    }

    pub fn render_single_pane_menu_rtl(&mut self, ui: &mut Ui, device: &Device) {
        self.render_language_selector(ui, device);
        ui.separator();
        if ui
            .add(
                Label::new(
                    RichText::new(egui_phosphor::light::SLIDERS).size(device.top_icon_size()),
                )
                .sense(Sense::click()),
            )
            .clicked()
        {
            self.core.select::<modules::Settings>();
        }

        ui.separator();
        if ui
            .add(
                Label::new(RichText::new(egui_phosphor::light::LOCK).size(device.top_icon_size()))
                    .sense(Sense::click()),
            )
            .clicked()
        {
            let wallet = self.core.wallet();
            spawn(async move {
                wallet.wallet_close().await?;
                Ok(())
            });
        }

        if self.core.notifications().has_some() {
            ui.separator();
            self.core.notifications().render(ui, device);
        }
    }

    pub fn render_desktop_menu_rtl(&mut self, ui: &mut Ui, device: &Device) {
        self.render_language_selector(ui, device);
        ui.separator();
        self.render_display_settings(ui, device);
        if self.core.notifications().has_some() {
            ui.separator();
            self.core.notifications().render(ui, device);
        }
    }

    pub fn render_desktop_menu(&mut self, ui: &mut Ui) {
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

    pub fn render_language_selector(&mut self, ui: &mut Ui, device: &Device) {
        let dictionary = i18n::dictionary();
        let lang_menu = if device.orientation() == Orientation::Portrait {
            RichText::new(TRANSLATE).size(device.top_icon_size())
            // RichText::new(TRANSLATE).size(18.)
        } else {
            RichText::new(format!("{} ⏷", dictionary.current_title()))
        };
        #[allow(clippy::useless_format)]
        ui.menu_button(lang_menu, |ui| {
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    let disable = ["ar","fa","he","hi","ja","ko","zh"];
                } else {
                    let disable = [];
                }
            }

            dictionary
                .enabled_languages()
                .into_iter()
                .filter(|(code, _)| !disable.contains(&code.as_str()))
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
    }

    pub fn render_display_settings(&mut self, ui: &mut Ui, device: &Device) {
        PopupPanel::new(
            PopupPanel::id(ui, "display_settings"),
            |ui| {
                ui.add(
                    // Label::new(RichText::new(egui_phosphor::light::MONITOR).size(16.))
                    Label::new(
                        RichText::new(egui_phosphor::light::MONITOR).size(device.top_icon_size()),
                    )
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
                                let theme_color = theme_color();
                                let current_theme_color_name = theme_color.name();
                                ui.menu_button(format!("{} ⏷", current_theme_color_name), |ui| {
                                    theme_colors().keys().for_each(|name| {
                                        if name.as_str() != current_theme_color_name
                                            && ui.button(name).clicked()
                                        {
                                            apply_theme_color_by_name(ui.ctx(), name);
                                            self.core.settings.user_interface.theme_color =
                                                name.to_string();
                                            self.core.store_settings();
                                            ui.close_menu();
                                        }
                                    });
                                });
                            });
                            ui.add_space(1.);
                        });
                        ui.end_row();

                        ui.label(i18n("Theme Style"));
                        ui.horizontal(|ui| {
                            let theme_style = theme_style();
                            let current_theme_style_name = theme_style.name();
                            ui.menu_button(format!("{} ⏷", current_theme_style_name), |ui| {
                                theme_styles().keys().for_each(|name| {
                                    if name.as_str() != current_theme_style_name
                                        && ui.button(name).clicked()
                                    {
                                        apply_theme_style_by_name(ui.ctx(), name);
                                        self.core.settings.user_interface.theme_style =
                                            name.to_string();
                                        self.core.store_settings();
                                        ui.close_menu();
                                    }
                                });
                            });
                        });
                        ui.end_row();

                        if runtime::is_native() {
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

                            response = response.on_hover_text(i18n("Capture a screenshot"));

                            if response.clicked() {
                                *_close_popup = true;
                                ui.ctx()
                                    .send_viewport_cmd(egui::ViewportCommand::Screenshot(
                                        UserData { data: None },
                                    ));
                            }
                        });
                    }
                }
            },
        )
        .with_min_width(64.)
        .build(ui);
    }
}
