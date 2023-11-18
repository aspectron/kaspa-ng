use crate::imports::*;

pub struct Send {
    #[allow(dead_code)]
    runtime: Runtime,
}

impl Send {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }
}

impl ModuleT for Send {
    fn render(
        &mut self,
        _core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        ui.heading("Send");
        ui.separator();
        ui.label("This is the send page");
    }
}
