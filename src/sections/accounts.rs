use crate::imports::*;

pub struct Accounts {

}

impl Render for Accounts {
    fn render(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Accounts");
            ui.separator();
            ui.label("This is the accounts page");
        });
    }
}