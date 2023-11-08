use crate::imports::*;

pub struct Request {
    #[allow(dead_code)]
    interop: Interop,
}

impl Request {
    pub fn new(interop: Interop) -> Self {
        Self { interop }
    }
}

impl ModuleT for Request {
    fn render(
        &mut self,
        _wallet: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        ui.heading("Request");
        ui.separator();
        ui.label("This is the payment request page");
    }
}
