use kaspa_metrics::Metric;

use crate::imports::*;

pub struct Metrics {
    #[allow(dead_code)]
    interop: Interop,
}

impl Metrics {
    pub fn new(interop: Interop) -> Self {
        Self { interop }
    }
}

impl ModuleT for Metrics {
    fn render(
        &mut self,
        wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        // wallet.style()

        ui.style_mut().text_styles = wallet.default_style.text_styles.clone();

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

        CollapsingHeader::new("Kaspa Node")
            .default_open(false)
            .show(ui, |ui| {
                // ui.label("This is the settings page");

                if let Some(metrics) = wallet.metrics.as_ref() {

                    ui.vertical(|ui| {

                        for metric in Metric::list().into_iter() {
                            
                            let value = metrics.get(&metric);
                            let caption = metrics.format(&metric, true);
                            
                            ui.horizontal(|ui| {
                                ui.label(caption);
                                ui.label(format!(" ... ({})", value));
                            });

                            // mutex!
                            let metrics_data = self.interop.kaspa_service().metrics_data();
                            let data = metrics_data.get(&metric).unwrap();
                            // test code
                            let len = 5;
                            let last = data.len();
                            let first = if last < len { 0 } else { last - len };
                            let samples = &data[first..last];
                            let text = samples.iter().map(|sample| format!("{}", sample)).collect::<Vec<_>>().join(", ");
                            ui.label(format!("[{text}]"));
                            ui.label(" ");
                        }
                    });
                }



            });

        // CollapsingHeader::new("RPC Protocol")
        //     .default_open(false)
        //     .show(ui, |ui| {
        //         ui.label("This is the settings page");
        //     });
    }
}
