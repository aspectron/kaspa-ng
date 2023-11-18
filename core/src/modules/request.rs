use crate::imports::*;

pub struct Request {
    #[allow(dead_code)]
    runtime: Runtime,
}

impl Request {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }
}

impl ModuleT for Request {
    fn render(
        &mut self,
        _core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        ui.heading("Request");
        ui.separator();
        ui.label("This is the payment request page");
    }
}
