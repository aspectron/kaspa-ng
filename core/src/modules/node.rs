use kaspa_rpc_core::RpcPeerInfo;

use crate::imports::*;
use crate::utils::format_duration;

pub struct Node {
    #[allow(dead_code)]
    interop: Interop,
}

impl Node {
    pub fn new(interop: Interop) -> Self {
        Self { interop }
    }
}

impl ModuleT for Node {

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Default
    }

    fn render(
        &mut self,
        wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        ui.heading(i18n("Node Status"));
        ui.separator();
        
        if !wallet.state().is_connected() {
            ui.label(i18n("Not connected"));
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])

            .show(ui, |ui| {


                CollapsingHeader::new(i18n("Network Peers"))
                    .default_open(true)
                    .show(ui, |ui| {

                        ui.vertical(|ui| {

                            if let Some(peers) = self.interop.kaspa_service().metrics().connected_peer_info() {
                                let (inbound, outbound) : (Vec<_>,Vec<_>) = peers.iter().partition(|peer| peer.is_outbound);

                                CollapsingHeader::new(i18n("Inbound"))
                                    .default_open(true)
                                    .show(ui, |ui| {

                                        inbound.iter().for_each(|peer| {
                                            render_peer(ui, peer);
                                        });
                                    });

                                CollapsingHeader::new(i18n("Outbound"))
                                    .default_open(true)
                                    .show(ui, |ui| {

                                        outbound.iter().for_each(|peer| {
                                            render_peer(ui, peer);
                                        });
                                    });
                            } else {
                                ui.colored_label(theme().warning_color, i18n("No peers"));
                            }

                        });

                    });
                });
            
    }
}

fn render_peer(ui : &mut Ui, peer: &RpcPeerInfo) {

    let color = theme().node_data_color;

    CollapsingHeader::new(peer.id.to_string())
        .default_open(true)
        .show(ui, |ui| {

            Grid::new("peer_info_grid")
                .num_columns(2)
                .spacing([40.0,4.0])
                .min_col_width(140.0)
                // .striped(true)
                .show(ui, |ui| {

                    ui.label(i18n("User Agent"));
                    ui.colored_label(color, peer.user_agent.to_string());
                    ui.end_row();

                    ui.label(i18n("Address"));
                    ui.colored_label(color, peer.address.to_string());
                    ui.end_row();

                    ui.label(i18n("Protocol"));
                    ui.colored_label(color, peer.advertised_protocol_version.to_string());
                    ui.end_row();

                    ui.label(i18n("Ping"));
                    ui.colored_label(color, format_duration(peer.last_ping_duration));
                    ui.end_row();

                    ui.label(i18n("IBD"));
                    ui.colored_label(color, peer.is_ibd_peer.to_string());
                    ui.end_row();

                    ui.label(i18n("Time Offset"));
                    ui.colored_label(color, peer.time_offset.to_string());
                    ui.end_row();

                    ui.label(i18n("Connection duration"));
                    ui.colored_label(color, format_duration(peer.time_connected));
                    ui.end_row();

                });

        });
}