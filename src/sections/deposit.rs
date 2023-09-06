use crate::imports::*;

pub struct Deposit {

}

impl Render for Deposit {
    fn render(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Deposit");
            ui.separator();
            ui.label("This is the deposit page");
        });
    }
}