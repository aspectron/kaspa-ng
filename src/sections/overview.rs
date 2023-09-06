use crate::imports::*;

pub struct Overview {

}

impl Render for Overview {
    fn render(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Overview");
            ui.separator();
            ui.label("This is the overview page");
        });
    }
}
