use crate::imports::*;

pub struct About {
    #[allow(dead_code)]
    runtime: Runtime,
}

impl About {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }

}

impl ModuleT for About {
    fn render(
        &mut self,
        _core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        _ui: &mut egui::Ui,
    ) {

    }
}
