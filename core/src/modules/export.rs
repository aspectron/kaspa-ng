use crate::imports::*;

#[derive(Clone)]
pub enum State {
    Select,
    Unlock(Option<String>),
    Unlocking,
}

pub struct Export {
    #[allow(dead_code)]
    interop: Interop,
    secret: String,
    pub state: State,
    pub message: Option<String>,

    selected_wallet: Option<String>,
}

impl Export {
    pub fn new(interop: Interop) -> Self {
        Self {
            interop,
            secret: String::new(),
            state: State::Select,
            message: None,
            selected_wallet: None,
        }
    }

    pub fn lock(&mut self) {
        self.state = State::Unlock(None);
    }
}

impl ModuleT for Export {
    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            let size = egui::Vec2::new(200_f32, 40_f32);
            let unlock_result = Payload::<Result<()>>::new("test");

            match self.state.clone() {
                State::Select => {
                    ui.heading("Select Wallet");
                    ui.label(" ");
                    ui.label("Select a wallet to unlock");
                    ui.label(" ");
                    // ui.add_space(32.);

                    egui::ScrollArea::vertical()
                        .id_source("wallet-list")
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());

                            for wallet in core.wallet_list.iter() {
                                if ui
                                    .add_sized(size, egui::Button::new(wallet.filename.clone()))
                                    .clicked()
                                {
                                    self.selected_wallet = Some(wallet.filename.clone());
                                    self.state = State::Unlock(None);
                                }
                            }
                        });
                }
                State::Unlock(message) => {
                    ui.heading("Unlock Wallet");

                    egui::ScrollArea::vertical()
                        .id_source("unlock-wallet")
                        .show(ui, |ui| {
                            ui.label(" ");
                            ui.label(format!(
                                "Opening wallet: \"{}\"",
                                self.selected_wallet.as_ref().unwrap()
                            ));
                            ui.label(" ");

                            if let Some(message) = message {
                                ui.label(" ");

                                // ui.label(format!("Error: {}",message));

                                ui.label(
                                    egui::RichText::new("Error unlocking wallet")
                                        .color(egui::Color32::from_rgb(255, 120, 120)),
                                );
                                ui.label(
                                    egui::RichText::new(message)
                                        .color(egui::Color32::from_rgb(255, 120, 120)),
                                );

                                ui.label(" ");
                            }

                            ui.label("Enter your password to unlock your wallet");
                            ui.label(" ");

                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut self.secret)
                                    .hint_text("Enter Password...")
                                    .password(true)
                                    .vertical_align(Align::Center),
                            );

                            if ui.add_sized(size, egui::Button::new("Unlock")).clicked() {
                                let secret = kaspa_wallet_core::secret::Secret::new(
                                    self.secret.as_bytes().to_vec(),
                                );
                                self.secret.zeroize();
                                let wallet = self.interop.wallet();//.clone();
                                let wallet_name = self.selected_wallet.clone(); //.expect("Wallet name not set");

                                spawn_with_result(&unlock_result, async move {
                                    wallet.wallet_open(secret, wallet_name, true).await?;
                                    Ok(())
                                });

                                self.state = State::Unlocking;
                            }

                            ui.label(" ");

                            if ui
                                .add_sized(size, egui::Button::new("Select a different wallet"))
                                .clicked()
                            {
                                self.state = State::Select;
                            }
                        });
                }
                State::Unlocking => {
                    ui.heading("Unlocking");
                    // ui.separator();
                    ui.label(" ");
                    ui.label("Unlocking wallet, please wait...");
                    ui.label(" ");
                    ui.add_space(64.);
                    ui.add(egui::Spinner::new().size(92.));

                    if let Some(result) = unlock_result.take() {
                        match result {
                            Ok(_) => {
                                println!("Unlock success");
                                // self.state = State::Unlock;
                                core.select::<modules::AccountManager>();
                            }
                            Err(err) => {
                                println!("Unlock error: {}", err);
                                self.state = State::Unlock(Some(err.to_string()));
                            }
                        }
                        // ui.label(format!("Result: {:?}", result));
                        // _ctx.value = result.unwrap();
                        // Stage::Next
                    } else {
                        // Stage::Current
                    }
                }
            }
        });
    }
}
