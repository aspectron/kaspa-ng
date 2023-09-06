use crate::imports::*;

pub struct Send {

}

impl Render for Send {
    fn render(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Send");
            ui.separator();
            ui.label("This is the send page");
        });
    }
}