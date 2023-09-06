use crate::imports::*;

pub struct Unlock {
    events : Channel<Events>,
    secret : String,
}

impl Render for Unlock {
    fn render(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Unlock");
            ui.separator();
            ui.label("This is the unlock page");

            ui.add(egui::Label::new("Password"));

            if ui.add(egui::Button::new("Unlock")).clicked() {
                let secret = Secret::new(self.secret.as_bytes().to_vec());
                unsafe { self.secret.as_mut_vec().iter_mut().for_each(|byte| *byte = 0); }
                self.secret.clear();
                self.events.try_send(Events::TryUnlock(secret)).unwrap();
            }
        });
    }
}