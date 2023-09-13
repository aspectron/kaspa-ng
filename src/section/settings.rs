use crate::imports::*;

pub struct Settings {
    #[allow(dead_code)]
    interop: Interop,
}

impl Settings {
    pub fn new(interop: Interop) -> Self {
        Self { interop }
    }
}

impl SectionT for Settings {
    fn render(
        &mut self,
        _wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        // ui.heading("Settings");
        // ui.separator();
        // ui.label("This is the settings page");

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
