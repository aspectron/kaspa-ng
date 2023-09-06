use crate::imports::*;

pub struct Deposit {
    sender : Sender<Events>,

}

impl Deposit {
    pub fn new(sender : Sender<Events>) -> Self {
        Self {
            sender,
        }
    }
}



impl SectionT for Deposit {
    fn render(&mut self, _wallet : &mut Wallet, _ctx: &egui::Context, _frame: &mut eframe::Frame, ui : &mut egui::Ui) {
        ui.heading("Deposit");
        ui.separator();
        ui.label("This is the deposit page");
    }
}