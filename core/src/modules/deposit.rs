use crate::imports::*;

pub struct Deposit {
    #[allow(dead_code)]
    interop: Interop,
}

impl Deposit {
    pub fn new(interop: Interop) -> Self {
        Self { interop }
    }
}

impl ModuleT for Deposit {
    fn render(
        &mut self,
        _core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        ui.heading("Deposit");
        ui.separator();
        ui.label("This is the deposit page");
    }
}
