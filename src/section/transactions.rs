use crate::imports::*;

pub struct Transactions {
    sender : Sender<Events>,

}

impl Transactions {
    pub fn new(sender : Sender<Events>) -> Self {
        Self {
            sender,
        }
    }
}



impl SectionT for Transactions {
    fn render(&mut self, _wallet : &mut Wallet, _ctx: &egui::Context, _frame: &mut eframe::Frame, ui : &mut egui::Ui) {
        ui.heading("Transactions");
        ui.separator();
        ui.label("This is the transactions page");
    }
}