use crate::imports::*;


// pub struct Config {
//     network: Network,
// }

pub struct Settings {
    #[allow(dead_code)]
    runtime: Runtime,
    settings : crate::settings::Settings,
    // pub kaspad: KaspadNodeKind,
    grpc_network_interface : NetworkInterfaceEditor, //::try_from(&self.settings.node.grpc_network_interface).unwrap();


}

impl Settings {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime, settings : crate::settings::Settings::default(),
        grpc_network_interface : NetworkInterfaceEditor::default(),
         }
    }

    pub fn load(&mut self, settings : crate::settings::Settings) {
        self.settings = settings;

        self.grpc_network_interface = NetworkInterfaceEditor::try_from(&self.settings.node.grpc_network_interface).unwrap();

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

        // self.grpc_network_interface = 

        // wallet.style()

        // ui.style_mut().text_styles = wallet.default_style.text_styles.clone();

        // ui.heading("Settings");
        // ui.separator();
        // ui.label("This is the settings page");

        // - wRPC JSON, wRPC BORSH, gRPC
        // - PUBLIC RPC LISTEN !
        // - MAINNET, TESTNET-10, TESTNET-11
        // - CUSTOM RPC PORTS
        // - EXTERNAL IP
        // -
        // ---
        // - pub connect_peers: Vec<ContextualNetAddress>,
        // - pub add_peers: Vec<ContextualNetAddress>,
        // - pub outbound_target: usize,
        // - pub inbound_limit: usize,
        // - pub rpc_max_clients: usize, gRPC
        // - pub enable_unsynced_mining: bool,
        // - pub enable_mainnet_mining: bool,
        // - pub perf_metrics: bool,
        // - pub perf_metrics_interval_sec: u64,


        CollapsingHeader::new("Kaspa p2p Network & Node Connection")
            .default_open(true)
            .show(ui, |ui| {

                CollapsingHeader::new("Kaspa Network")
                    .default_open(true)
                    .show(ui, |ui| {
                        // ui.label("This is the settings page");

                        ui.horizontal_wrapped(|ui|{

                            Network::iter().for_each(|network| {
                                ui.radio_value(&mut self.settings.node.network, *network, network.to_string());
                            });

                            // ui.radio_value(&mut self.settings.network, Network::Mainnet, "MAINNET");
                            // ui.radio_value(&mut self.settings.network, Network::Testnet10, "TESTNET-10");
                            // ui.radio_value(&mut self.settings.network, Network::Testnet11, "TESTNET-11");
                        });


                    });


                CollapsingHeader::new("Kaspa Node")
                    .default_open(true)
                    .show(ui, |ui| {

                    // ui.label

                        ui.horizontal_wrapped(|ui|{

                            KaspadNodeKind::iter().for_each(|node_kind| {
                                ui.radio_value(&mut self.settings.node.node_kind, *node_kind, node_kind.to_string());
                            });
                            // ui.radio_value(&mut self.settings.kaspad, KaspadNodeKind::Remote, "Remote");
                            // cfg_if! {
                            //     if #[cfg(not(target_arch = "wasm32"))] {
                            //         ui.radio_value(&mut self.settings.kaspad, KaspadNodeKind::IntegratedInProc, "Internal");
                            //         ui.radio_value(&mut self.settings.kaspad, KaspadNodeKind::IntegratedAsDaemon, "Internal Daemon");
                            //         ui.radio_value(&mut self.settings.kaspad, KaspadNodeKind::ExternalAsDaemon, "External Daemon");
                            //     }
                            // }
                        });

                        // ui.label("")
                            
                        ui.label("This is the settings page");

                    });

                    CollapsingHeader::new("Client RPC")
                        .default_open(true)
                        .show(ui, |ui| {
                    // ui.horizontal(|ui|{
                    //     ui.separator();

                            ui.vertical(|ui|{
                                ui.checkbox(&mut self.settings.node.enable_grpc, i18n("Enable gRPC"));
                                if self.settings.node.enable_grpc {

                                    CollapsingHeader::new("gRPC Network Interface & Port")
                                        .default_open(true)
                                        .show(ui, |ui| {
                                        // ui.horizontal(|ui|{
                                        //     ui.separator();
                    
                                            // ui.label(i18n("gRPC Network Interface & Port: "));
                                            self.grpc_network_interface.ui(ui);
                                        });
                                    // ui.add(TextEdit::singleline(&mut self.settings.node.grpc_network_interface));
                                }
                            });

                    });
                    
                CollapsingHeader::new("p2p RPC")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.vertical(|ui|{
                            ui.checkbox(&mut self.settings.node.enable_upnp, i18n("Enable UPnP"));
                        });
                    });

                ui.add_space(32.);
                ui.horizontal(|ui|{
                    ui.label("URL: ");
                    ui.add(TextEdit::singleline(&mut self.settings.node.wrpc_url));
                });
                ui.horizontal_wrapped(|_ui|{
                    // ui.radio_value(&mut );
                });

                ui.label("This is the settings page");


                if !self.grpc_network_interface.is_valid() {
                    return;
                } else {
                    self.settings.node.grpc_network_interface = self.grpc_network_interface.as_ref().try_into().unwrap(); //NetworkInterfaceConfig::try_from(&self.grpc_network_interface).unwrap();
                }


                if let Some(restart) = self.settings.node.compare(&core.settings.node) {

                    if let Some(response) = ui.confirm_medium_apply_cancel(Align::Max) {
                        match response {
                            Confirm::Ack => {


                                core.settings = self.settings.clone();
                                core.settings.store_sync().unwrap();
                                if restart {
                                    println!("NODE INTERFACE UPDATE: {:?}", self.settings.node);
                                    self.runtime.kaspa_service().update_services(&self.settings.node);
                                    // println!("TODO - restart");
                                }
                            },
                            Confirm::Nack => {
                                self.settings = core.settings.clone();
                            }
                        }
                    }

                    ui.separator();
                }

            });

            if ui.button("Test Toast").clicked() {
                self.runtime.try_send(Events::Notify {
                    notification : Notification::info("Test Toast")
                }).unwrap();
            }
            ui.add_space(32.);
            if ui.button("Test Panic").clicked() {
                panic!("Testing panic...");
            }
    }
}

// if let Some(result) = spawn!(async move {

//     println!("Spawn executing...");
//     Ok(123)
// }) {

//     println!("Result {:?}", result);
//     ui.label(format!("Result {:?}", result));
// }
