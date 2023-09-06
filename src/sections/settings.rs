use crate::imports::*;

pub struct Settings {

}

impl Render for Settings {
    fn render(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();
            ui.label("This is the settings page");
        });
    }
}