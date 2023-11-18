use crate::imports::*;

pub struct Deposit {
    #[allow(dead_code)]
    runtime: Runtime,
}

impl Deposit {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
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
