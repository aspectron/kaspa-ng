use crate::imports::*;
use crate::sync::SyncStatus;
// use kaspa_metrics_core::MetricsSnapshot;

enum ConnectionStatus {
    Connected {
        current_daa_score: Option<u64>,
        peers: Option<usize>,
        #[allow(dead_code)]
        tps: Option<f64>,
    },
    Disconnected,
    Syncing {
        sync_status: Option<SyncStatus>,
        peers: Option<usize>,
    },
    Error {
        error: String,
    },
}

pub struct Status<'core> {
    core: &'core mut Core,
}

impl<'core> Status<'core> {
    pub fn new(core: &'core mut Core) -> Self {
        Self { core }
    }

    fn state(&self) -> &State {
        self.core.state()
    }

    fn settings(&self) -> &Settings {
        &self.core.settings
    }

    fn module(&self) -> &Module {
        self.core.module()
    }

    fn device(&mut self) -> &Device {
        self.core.device()
    }

    // fn metrics(&self) -> &Option<Box<MetricsSnapshot>> {
    //     self.core.metrics()
    // }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        menu::bar(ui, |ui| {
            if !self.state().is_connected() {
                self.render_connected_state(ui, ConnectionStatus::Disconnected);
            } else {
                let peers = self.state().peers();
                let tps = self.state().tps();

                ui.horizontal(|ui| {
                    if let Some(error) = self.state().error() {
                        self.render_connected_state(
                            ui,
                            ConnectionStatus::Error {
                                error: error.clone(),
                            },
                        );
                    } else if self.state().is_synced() {
                        self.render_connected_state(
                            ui,
                            ConnectionStatus::Connected {
                                current_daa_score: self.state().current_daa_score(),
                                peers,
                                tps,
                            },
                        );
                    } else {
                        self.render_connected_state(
                            ui,
                            ConnectionStatus::Syncing {
                                sync_status: self
                                    .state()
                                    .sync_state
                                    .as_ref()
                                    .map(SyncStatus::try_from),
                                peers,
                            },
                        );
                    }
                });
            }
        });
    }

    fn render_peers(&self, ui: &mut egui::Ui, peers: Option<usize>) {
        let status_icon_size = theme_style().status_icon_size;

        if let Some(peers) = peers {
            if peers != 0 {
                let text = if peers > 1 {
                    i18n("peers")
                } else {
                    i18n("peer")
                };
                ui.label(format!("{peers} {text}"));
            } else {
                ui.label(
                    RichText::new(egui_phosphor::light::CLOUD_SLASH)
                        .size(status_icon_size)
                        .color(theme_color().error_color),
                );
                ui.label(RichText::new(i18n("No peers")).color(theme_color().error_color));
            }
        } else {
            ui.label(RichText::new(egui_phosphor::light::CLOUD_ARROW_DOWN).size(status_icon_size));
            ui.label(RichText::new("..."));
        }
    }

    fn render_connection_selector(&mut self, ui: &mut Ui) {
        let connection_selector = !self.core.module().modal()
            && self.core.settings.node.connection_config_kind.is_public();

        if !connection_selector {
            ui.label(i18n("CONNECTED")).on_hover_ui(|ui| {
                if let Some(wrpc_url) = runtime().kaspa_service().rpc_url() {
                    ui.horizontal(|ui| {
                        ui.label(wrpc_url);
                    });
                }
            });
        } else {
            let response =
                ui.add(Label::new(RichText::new(i18n("CONNECTED"))).sense(Sense::click()));

            let popup_id = PopupPanel::id(ui, "node_connection_selector_popup");

            if !PopupPanel::is_open(ui, popup_id) {
                response.on_hover_ui(|ui| {
                    if let Some(wrpc_url) = runtime().kaspa_service().rpc_url() {
                        ui.horizontal(|ui| {
                            ui.label(wrpc_url);
                        });
                    }
                });
            }
        }
    }

    fn render_network_selector(&mut self, ui: &mut Ui) {
        use egui_phosphor::light::CHECK;

        let network_selector = !self.core.module().modal();

        if !network_selector {
            ui.label(self.settings().node.network.to_string());
        } else {
            let response = ui.add(
                Label::new(RichText::new(self.settings().node.network.to_string()))
                    .sense(Sense::click()),
            );
            let id = PopupPanel::id(ui, "network_selector_popup");
            PopupPanel::new(
                id,
                |_ui| response,
                |ui, close| {
                    set_menu_style(ui.style_mut());

                    Network::iter().for_each(|network| {
                        let name = if *network == self.settings().node.network {
                            format!("{network} {CHECK}")
                        } else {
                            network.to_string()
                        };

                        if ui.button(name).clicked() {
                            *close = true;
                            self.core.change_current_network(*network);
                        }
                    });
                },
            )
            .with_min_width(100.0)
            .with_above_or_below(AboveOrBelow::Above)
            .build(ui);
        }
    }

    fn render_separator(&mut self, ui: &mut egui::Ui) {
        if self.device().desktop() {
            ui.separator();
        } else {
            Separator::default().spacing(1.0).ui(ui);
        }
    }
    fn render_connected_state(&mut self, ui: &mut egui::Ui, state: ConnectionStatus) {
        let status_area_width = ui.available_width() - 24.;
        let status_icon_size = theme_style().status_icon_size;
        let module = self.module().clone();
        let left_padding = if self.device().desktop() { 8.0 } else { 5.0 };

        match state {
            ConnectionStatus::Disconnected => {
                ui.add_space(left_padding);

                match self.settings().node.node_kind {
                    KaspadNodeKind::Disable => {
                        ui.label(
                            RichText::new(egui_phosphor::light::PLUGS)
                                .size(status_icon_size)
                                .color(theme_color().error_color),
                        );
                        ui.separator();
                        ui.label(i18n("Not Connected"));
                    }
                    KaspadNodeKind::Remote => {
                        ui.label(
                            RichText::new(egui_phosphor::light::CLOUD_X)
                                .size(status_icon_size)
                                .color(theme_color().error_color),
                        );
                        ui.separator();

                        let settings = self.settings();
                        match settings.node.node_kind {
                            KaspadNodeKind::Remote => match settings.node.connection_config_kind {
                                NodeConnectionConfigKind::Custom => {
                                    match KaspaRpcClient::parse_url(
                                        settings.node.wrpc_url.clone(),
                                        settings.node.wrpc_encoding,
                                        settings.node.network.into(),
                                    ) {
                                        Ok(url) => {
                                            ui.label(format!(
                                                "{} {} ...",
                                                i18n("Connecting to"),
                                                url
                                            ));
                                        }
                                        Err(err) => {
                                            ui.label(
                                                RichText::new(format!(
                                                    "{} {}: {err}",
                                                    i18n("Error connecting to"),
                                                    settings.node.wrpc_url
                                                ))
                                                .color(theme_color().warning_color),
                                            );
                                        }
                                    }
                                }
                                NodeConnectionConfigKind::PublicServerCustom => {
                                    if let Some(rpc_url) = runtime().kaspa_service().rpc_url() {
                                        ui.label(format!(
                                            "{} {} ...",
                                            i18n("Connecting to"),
                                            rpc_url
                                        ));
                                    }
                                }
                                NodeConnectionConfigKind::PublicServerRandom => {
                                    if let Some(instant) = runtime()
                                        .kaspa_service()
                                        .services_start_instant
                                        .lock()
                                        .unwrap()
                                        .as_ref()
                                    {
                                        let elapsed = instant.elapsed();
                                        if elapsed.as_millis() > 6_500 {
                                            if ui
                                                .add(
                                                    Label::new(RichText::new(i18n(
                                                        "Click to try another server...",
                                                    )))
                                                    .sense(Sense::click()),
                                                )
                                                .clicked()
                                            {
                                                let options = runtime()
                                                    .kaspa_service()
                                                    .rpc_url()
                                                    .map(|rpc_url| {
                                                        RpcOptions::new().blacklist(rpc_url)
                                                    });

                                                runtime().kaspa_service().update_services(
                                                    &self.core.settings.node,
                                                    options,
                                                );
                                            }

                                            ui.separator();
                                        }
                                    }

                                    if let Some(rpc_url) = runtime().kaspa_service().rpc_url() {
                                        ui.label(format!(
                                            "{} {} ...",
                                            i18n("Connecting to"),
                                            rpc_url
                                        ));

                                        ui.ctx().request_repaint_after(Duration::from_millis(250));
                                    }
                                }
                            },
                            _ => {
                                ui.label(i18n("Connecting..."));
                            }
                        }
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    _ => {
                        ui.vertical(|ui| {
                            ui.add_space(2.);
                            ui.add(egui::Spinner::new());
                        });
                        // ui.label(
                        //     RichText::new(egui_phosphor::light::PLUGS)
                        //         .size(status_icon_size)
                        //         .color(Color32::LIGHT_RED),
                        // );
                        ui.separator();
                        ui.label(i18n("Starting..."));
                    }
                }

                if !self.device().single_pane() {
                    module.status_bar(self.core, ui);
                }
            }

            ConnectionStatus::Connected {
                current_daa_score,
                peers,
                tps: _,
            } => {
                ui.add_space(left_padding);

                if peers.is_some() {
                    cfg_if! {
                        if #[cfg(target_arch = "wasm32")] {
                            let icon = egui_phosphor::light::CPU;
                        } else {
                            let icon = if self.core.settings.node.node_kind.is_local() {
                                egui_phosphor::light::CPU
                            } else {
                                egui_phosphor::light::CLOUD
                            };
                        }
                    }

                    ui.label(
                        // "\u{E610}"
                        RichText::new(icon)
                            .size(status_icon_size)
                            .family(FontFamily::Name("phosphor".into()))
                            .color(theme_color().icon_connected_color),
                    );
                } else {
                    ui.add(egui::Spinner::new());
                }
                self.render_separator(ui);
                self.render_connection_selector(ui);
                self.render_separator(ui);
                self.render_network_selector(ui);

                if !self.device().mobile() {
                    self.render_separator(ui);
                    self.render_peers(ui, peers);
                    if let Some(current_daa_score) = current_daa_score {
                        self.render_separator(ui);
                        ui.label(format!("DAA {}", current_daa_score.separated_string()));
                    }
                }

                if !self.device().single_pane() {
                    module.status_bar(self.core, ui);
                }
            }
            ConnectionStatus::Syncing { sync_status, peers } => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(left_padding);

                        if peers.is_some() {
                            ui.label(
                                RichText::new(egui_phosphor::light::CLOUD_ARROW_DOWN)
                                    .size(status_icon_size)
                                    .color(theme_color().icon_syncing_color),
                            );
                        } else {
                            ui.add(egui::Spinner::new());
                        }

                        ui.separator();
                        self.render_connection_selector(ui);
                        ui.separator();
                        self.render_network_selector(ui);

                        if !self.device().single_pane() {
                            ui.separator();
                            self.render_peers(ui, peers);
                            if let Some(status) = sync_status.as_ref() {
                                if !status.synced {
                                    ui.separator();
                                    status.render_text_state(ui);
                                }
                            }

                            module.status_bar(self.core, ui);
                        }
                    });

                    if let Some(status) = sync_status.as_ref() {
                        if !status.synced {
                            status
                                .progress_bar(ui)
                                .map(|bar| ui.add(bar.desired_width(status_area_width)));
                        }
                    }
                });
            }

            ConnectionStatus::Error { error } => {
                ui.add_space(left_padding);

                ui.label(
                    RichText::new(egui_phosphor::light::SEAL_WARNING)
                        .size(status_icon_size)
                        .color(theme_color().error_color),
                );
                ui.separator();
                ui.label(i18n(error.as_str()));
            }
        }
    }
}
