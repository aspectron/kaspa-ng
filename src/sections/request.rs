use crate::imports::*;

pub struct Request {

}

impl Render for Request {
    fn render(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Request");
            ui.separator();
            ui.label("This is the payment request page");
        });
    }
}
