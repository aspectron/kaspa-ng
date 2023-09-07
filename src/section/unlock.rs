use crate::imports::*;
use egui::*;
use workflow_core::task::spawn;

pub enum State {
    Locked,
    Unlocking,
}

pub struct Unlock {
    sender : Sender<Events>,
    secret : String,
    pub state : State,
    pub message: Option<String>,
}

impl Unlock {
    pub fn new(sender : Sender<Events>) -> Self {
        Self {
            sender,
            secret : String::new(),
            state : State::Locked,
            message: None,
        }
    }

    pub fn lock(&mut self) {
        self.state = State::Locked;
    }

    fn render_unlocking(&mut self, ui : &mut Ui, wallet : &mut Wallet) {
        ui.heading("Unlocking");
        ui.separator();
        ui.label("Unlocking wallet, please wait...");
    }

    fn render_locked(&mut self, ui : &mut Ui, wallet : &mut Wallet) {
        let size = egui::Vec2::new(200_f32,40_f32);

        if let Some(message) = &self.message {
            


            
            
            ui.label(" ");
            // ui.add(egui::Label::new(message));
            // ui.label(egui::RichText::new(message).heading().color(egui::Color32::from_rgb(255, 255, 255)));
            // ui.label(egui::RichText::new(message).heading().color(egui::Color32::from_rgb(255, 128, 128)));
            ui.label(egui::RichText::new(message).color(egui::Color32::from_rgb(255, 128, 128)));
            ui.label(" ");
        }

        // ui.add(egui::Label::new("Password"));
        ui.label(" ");
        ui.label(" ");

        ui.add_sized(size, TextEdit::singleline(&mut self.secret).hint_text("Enter Password...").password(true).vertical_align(Align::Center));
        
        // ui.add_sized(egui::Vec2::new(120_f32,40_f32), egui::Button::new("Testing 123"));
        
        if ui.add_sized(size, egui::Button::new("Unlock")).clicked() {
            println!("secret: {}", self.secret);
            let secret = kaspa_wallet_core::secret::Secret::new(self.secret.as_bytes().to_vec());
            unsafe { self.secret.as_mut_vec().iter_mut().for_each(|byte| *byte = 0); }
            self.secret.clear();
            self.state = State::Unlocking;
            // self.sender.try_send(Events::TryUnlock(secret.into())).unwrap();

            let sender = wallet.sender();
            let wallet = wallet.wallet().clone();
            // wallet.spawn(|wallet| {
            // });

            spawn(async move {
                match wallet.load(secret,None).await {
                    Ok(_) => {
                        sender.send(Events::UnlockSuccess).await.unwrap();
                    },
                    Err(err) => {
                        sender.send(Events::UnlockFailure { message : err.to_string() }).await.unwrap();
                    }
                }
            });
            
            // let channel = wallet.spawn(wallet.wallet().load(secret, None));

        }
    }

    
}

impl SectionT for Unlock {
    fn render(&mut self, wallet : &mut Wallet, _ctx: &egui::Context, _frame: &mut eframe::Frame, ui : &mut egui::Ui) {

            // ui.horizontal_centered(|ui| {

            //     ui.heading("Unlock");
            // });
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Unlock your wallet");
                ui.separator();
                match self.state {
                    State::Locked => self.render_locked(ui, wallet),
                    State::Unlocking => self.render_unlocking(ui, wallet),
                }
            });
    }
}