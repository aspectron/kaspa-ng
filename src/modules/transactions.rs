use crate::imports::*;

pub struct Transactions {
    #[allow(dead_code)]
    interop: Interop,
}

impl Transactions {
    pub fn new(interop: Interop) -> Self {
        Self { interop }
    }
}

impl ModuleT for Transactions {
    fn render(
        &mut self,
        _wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        ui.heading("Transactions");
        ui.separator();
        ui.label("This is the transactions page");
    }
}
