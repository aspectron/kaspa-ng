use crate::imports::*;

pub struct Accounts {
    sender : Sender<Events>,
}

impl Accounts {
    pub fn new(sender : Sender<Events>) -> Self {
        Self {
            sender,
        }
    }
}

impl SectionT for Accounts {
    fn render(&mut self, _wallet : &mut Wallet, _ctx: &egui::Context, _frame: &mut eframe::Frame, ui : &mut egui::Ui) {
    // fn render(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, ui : &mut Ui) {
        ui.heading("Accounts");
        ui.separator();
        ui.label("This is the accounts page");
    }
}