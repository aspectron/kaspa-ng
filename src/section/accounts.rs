use crate::imports::*;

pub struct Accounts {
    #[allow(dead_code)]
    interop: Interop,
}

impl Accounts {
    pub fn new(interop: Interop) -> Self {
        Self { interop }
    }
}

impl SectionT for Accounts {
    fn main(
        &mut self,
        _wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        // fn render(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, ui : &mut Ui) {
        ui.heading("Accounts");
        ui.separator();
        ui.label("This is the accounts page");
    }
}
