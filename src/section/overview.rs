use crate::imports::*;

pub struct Overview {
    #[allow(dead_code)]
    interop: Interop,
}

impl Overview {
    pub fn new(interop: Interop) -> Self {
        Self { interop }
    }
}

impl SectionT for Overview {
    fn main(
        &mut self,
        _wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        ui.heading("Overview");
        ui.separator();
        ui.label("This is the overview page");
    }
}
