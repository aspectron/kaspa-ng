use kaspa_rpc_core::RpcPeerInfo;

use crate::imports::*;
use crate::utils::format_duration;

pub struct Node {
    #[allow(dead_code)]
    runtime: Runtime,
}

impl Node {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }
}

impl ModuleT for Node {

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

        ui.heading(i18n("Node Status"));
        ui.separator();
        
        if !core.state().is_connected() {
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

                            if let Some(peers) = self.runtime.peer_monitor_service().peer_info() {
                                let (outbound, inbound) : (Vec<_>,Vec<_>) = peers.iter().partition(|peer| peer.is_outbound);

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
                            } else if core.state().metrics().as_ref().map(|m| m.data.node_active_peers).unwrap_or_default() > 0 {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(i18n("Updating..."));
                                });
                            } else {
                                ui.colored_label(theme_color().warning_color, i18n("No peers"));
                            }

                        });

                    });
                });
            
    }

    fn activate(&mut self, _core: &mut Core) {
        crate::runtime::runtime().peer_monitor_service().enable();
    }

    fn deactivate(&mut self, _core: &mut Core) {
        crate::runtime::runtime().peer_monitor_service().disable();
    }

}

fn render_peer(ui : &mut Ui, peer: &RpcPeerInfo) {

    let color = theme_color().node_data_color;

    CollapsingHeader::new(peer.id.to_string())
        .default_open(true)
        .show(ui, |ui| {

            Grid::new("peer_info_grid")
                .num_columns(2)
                .spacing([16.0,4.0])
                .show(ui, |ui| {

                    ui.label(i18n("User Agent"));
                    ui.colored_label(color, peer.user_agent.to_string());
                    ui.end_row();

                    ui.label(i18n("Connection"));
                    ui.horizontal(|ui| {
                        ui.label(i18n("Address:"));
                        ui.colored_label(color, peer.address.to_string());
                        ui.label(i18n("Protocol:"));
                        ui.colored_label(color, format!("v{}", peer.advertised_protocol_version));
                        ui.label(i18n("IBD:"));
                        ui.colored_label(color, peer.is_ibd_peer.to_string());
                    });
                    ui.end_row();

                    ui.label(i18n("Metrics"));
                    ui.horizontal(|ui|{
                        ui.label(i18n("Ping:"));
                        ui.colored_label(color, format_duration(peer.last_ping_duration));
                        ui.label(i18n("Time Offset:"));
                        ui.colored_label(color, peer.time_offset.to_string());
                        ui.label(i18n("Uptime:"));
                        ui.colored_label(color, format_duration(peer.time_connected));
                    });
                    ui.end_row();
                });

        });
}