use crate::imports::*;
use crate::sync::SyncStatus;
use kaspa_metrics_core::MetricsSnapshot;

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

    fn metrics(&self) -> &Option<Box<MetricsSnapshot>> {
        self.core.metrics()
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        menu::bar(ui, |ui| {
            if !self.state().is_connected() {
                self.render_connected_state(ui, ConnectionStatus::Disconnected);
            } else {
                let peers = self
                    .metrics()
                    .as_ref()
                    .map(|metrics| metrics.data.node_active_peers as usize);

                let tps = self
                    .metrics()
                    .as_ref()
                    .map(|metrics| metrics.network_transactions_per_second);
                ui.horizontal(|ui| {
                    if self.state().is_synced() {
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

        let peers = peers.unwrap_or(0);
        if peers != 0 {
            ui.label(format!("{} peers", peers));
        } else {
            ui.label(
                RichText::new(egui_phosphor::light::CLOUD_SLASH)
                    .size(status_icon_size)
                    .color(theme_color().error_color),
            );
            ui.label(RichText::new("No peers").color(theme_color().error_color));
        }
    }

    fn render_network_selector(&self, ui: &mut Ui) {
        ui.label(self.settings().node.network.to_string());
        // ui.menu_button(self.settings.node.network.to_string(), |ui| {
        //     Network::iter().for_each(|network| {
        //         if ui.button(network.to_string()).clicked() {
        //             ui.close_menu();
        //         }
        //     });
        // });
    }

    fn render_connected_state(&mut self, ui: &mut egui::Ui, state: ConnectionStatus) {
        let status_area_width = ui.available_width() - 24.;
        let status_icon_size = theme_style().status_icon_size;
        let module = self.module().clone();
        let left_padding = 8.0;

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
                        // ui.label("Connecting...");

                        let settings = self.settings();
                        match settings.node.node_kind {
                            KaspadNodeKind::Remote => {
                                match KaspaRpcClient::parse_url(
                                    settings.node.wrpc_url.clone(),
                                    settings.node.wrpc_encoding,
                                    settings.node.network.into(),
                                ) {
                                    Ok(url) => {
                                        ui.label(format!("Connecting to {} ...", url));
                                    }
                                    Err(err) => {
                                        ui.label(
                                            RichText::new(format!(
                                                "Error connecting to {}: {err}",
                                                settings.node.wrpc_url
                                            ))
                                            .color(theme_color().warning_color),
                                        );
                                    }
                                }
                            }
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
                ui.label(
                    RichText::new(egui_phosphor::light::CPU)
                        .size(status_icon_size)
                        .color(theme_color().icon_connected_color),
                );
                ui.separator();
                ui.label(i18n("CONNECTED")).on_hover_ui(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(self.settings().node.wrpc_url.clone());
                    });
                });
                // }
                ui.separator();
                self.render_network_selector(ui);
                // ui.menu_button(self.settings.node.network.to_string(), |ui| {
                //     Network::iter().for_each(|network| {
                //         if ui.button(network.to_string()).clicked() {
                //             ui.close_menu();
                //         }
                //     });
                // });

                if !self.device().mobile() {
                    ui.separator();
                    self.render_peers(ui, peers);
                    if let Some(current_daa_score) = current_daa_score {
                        ui.separator();
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
                        ui.label(
                            RichText::new(egui_phosphor::light::CLOUD_ARROW_DOWN)
                                .size(status_icon_size)
                                .color(theme_color().icon_syncing_color),
                        );
                        ui.separator();
                        ui.label(i18n("CONNECTED")).on_hover_ui(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(self.settings().node.wrpc_url.clone());
                            });
                        });
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
        }
    }
}
