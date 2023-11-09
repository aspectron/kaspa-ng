use crate::imports::*;

pub struct About {
    #[allow(dead_code)]
    interop: Interop,
}

impl About {
    pub fn new(interop: Interop) -> Self {
        Self { interop }
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
