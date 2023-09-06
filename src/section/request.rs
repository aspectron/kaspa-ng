use crate::imports::*;

pub struct Request {
    sender : Sender<Events>,

}

impl Request {
    pub fn new(sender : Sender<Events>) -> Self {
        Self {
            sender,
        }
    }
}


impl SectionT for Request {
    fn render(&mut self, _wallet : &mut Wallet, _ctx: &egui::Context, _frame: &mut eframe::Frame, ui : &mut egui::Ui) {
        ui.heading("Request");
        ui.separator();
        ui.label("This is the payment request page");
    }
}
