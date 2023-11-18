use crate::imports::*;

pub struct Transactions {
    #[allow(dead_code)]
    runtime: Runtime,
}

impl Transactions {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }
}

impl ModuleT for Transactions {
    fn render(
        &mut self,
        _core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        ui.heading("Transactions");
        ui.separator();
        ui.label("This is the transactions page");
    }
}
