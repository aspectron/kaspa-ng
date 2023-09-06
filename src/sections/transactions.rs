use crate::imports::*;

pub struct Transactions {

}

impl Render for Transactions {
    fn render(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Transactions");
            ui.separator();
            ui.label("This is the transactions page");
        });
    }
}