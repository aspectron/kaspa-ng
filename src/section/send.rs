use crate::imports::*;

pub struct Send {
    #[allow(dead_code)]
    interop: Interop,
}

impl Send {
    pub fn new(interop: Interop) -> Self {
        Self { interop }
    }
}

impl SectionT for Send {
    fn render(
        &mut self,
        _wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        ui.heading("Send");
        ui.separator();
        ui.label("This is the send page");
    }
}
