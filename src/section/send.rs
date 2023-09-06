use crate::imports::*;

pub struct Send {
    sender : Sender<Events>,

}

impl Send {
    pub fn new(sender : Sender<Events>) -> Self {
        Self {
            sender,
        }
    }
}



impl SectionT for Send {
    fn render(&mut self, _wallet : &mut Wallet, _ctx: &egui::Context, _frame: &mut eframe::Frame, ui : &mut egui::Ui) {
        ui.heading("Send");
        ui.separator();
        ui.label("This is the send page");
    }
}