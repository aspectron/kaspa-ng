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

impl SectionT for Metrics {
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
                ui.label("This is the settings page");
            });
        CollapsingHeader::new("RPC Protocol")
            .default_open(false)
            .show(ui, |ui| {
                ui.label("This is the settings page");
            });
    }
}
