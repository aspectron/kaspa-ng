use crate::imports::*;
use kaspa_bip32::Language;

#[derive(Clone)]
pub enum State {
    Words,
    Select,
    Unlock(Option<String>),
    Unlocking,
}

pub struct Import {
    #[allow(dead_code)]
    runtime: Runtime,
    wallet_secret: String,

    word : String,
    mnemonic : Vec<String>,

    pub state: State,
    pub message: Option<String>,

    selected_wallet: Option<String>,
}

impl Import {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            wallet_secret: String::new(),

            word : String::new(),
            mnemonic : Vec::new(),

            state: State::Words,
            message: None,
            selected_wallet: None,
        }
    }

    pub fn lock(&mut self) {
        self.state = State::Unlock(None);
    }
}

impl ModuleT for Import {
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
                State::Words => {

                    Panel::new(self)
                        .with_caption(i18n("Mnemonic Import"))
                        .with_close_enabled(false, |_|{
                        })
                        .with_header(|this,ui| {
                            // ui.add_space(64.);
                            ui.label(i18n("Importing word N/N"));

                            ui.horizontal(|ui|{
                            // ui.vertical_centered_justified(|ui|{

                                // ui.label(this.mnemonic.last().unwrap_or(&String::new()));
                                this.mnemonic.iter().for_each(|word| {
                                    ui.label(" ");
                                    ui.label(RichText::new(word).family(FontFamily::Monospace).size(14.).color(egui::Color32::WHITE));
                                });
                            });
                            // ui.label(" ");
                            ui.separator();

                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut this.word)
                                    // .hint_text(format!("Enter Word {}...", this.mnemonic.len()+1))
                                    .hint_text(i18n_args("Enter Word {number}...", &[("number", &(this.mnemonic.len() + 1).to_string())]))
                                    .horizontal_align(Align::Center)
                                    // .vertical_align(Align::Center),
                            );

                            ui.label(" ");

                            // ui.label("A wallet is stored in a file on your computer. You can create multiple wallet.");
                        })
                        .with_body(|this, ui|{

                            let filter = this.word.clone();
                            let words = Language::English.wordlist();
                            words.iter().filter(|w|w.starts_with(filter.as_str())).for_each(|word| {  
                                if ui.large_button(word).clicked() {
                                    // - TODO - CAPTURE WORD
                                    this.mnemonic.push(word.to_string());
                                    this.word.clear();
                                }
                            });

                        })
                        // .with_footer(|_this,ui| {
                        //     // if ui.add_sized(theme().large_button_size, egui::Button::new(i18n("Continue"))).clicked() {
                        //     let size = theme().large_button_size;
                        //     if ui.add_sized(size, egui::Button::new(i18n("Continue"))).clicked() {
                        //         // this.state = State::WalletName;
                        //     }
                        // })
                        .render(ui);

                }
                State::Select => {
                    ui.heading(i18n("Select Wallet"));
                    ui.label(" ");
                    ui.label(i18n("Select a wallet to unlock"));
                    ui.label(" ");
                    // ui.add_space(32.);

                    egui::ScrollArea::vertical()
                        .id_salt("wallet-list")
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
                    ui.heading(i18n("Unlock Wallet"));

                    egui::ScrollArea::vertical()
                        .id_salt("unlock-wallet")
                        .show(ui, |ui| {
                            ui.label(" ");
                            ui.label(i18n_args("Opening wallet: {wallet}", &[("wallet", self.selected_wallet.as_ref().unwrap())]));
                            ui.label(" ");

                            if let Some(message) = message {
                                ui.label(" ");

                                ui.label(
                                    RichText::new(i18n("Error unlocking wallet"))
                                        .color(egui::Color32::from_rgb(255, 120, 120)),
                                );
                                ui.label(
                                    RichText::new(message)
                                        .color(egui::Color32::from_rgb(255, 120, 120)),
                                );

                                ui.label(" ");
                            }

                            ui.label(i18n("Enter your password to unlock your wallet"));
                            ui.label(" ");

                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut self.wallet_secret)
                                    .hint_text(i18n("Enter Password..."))
                                    .password(true)
                                    .vertical_align(Align::Center),
                            );

                            if ui.add_sized(size, egui::Button::new(i18n("Unlock"))).clicked() {
                                let wallet_secret = Secret::new(
                                    self.wallet_secret.as_bytes().to_vec()
                                );
                                self.wallet_secret.zeroize();
                                let wallet = self.runtime.wallet().clone();
                                let wallet_name = self.selected_wallet.clone(); //.expect("Wallet name not set");
                                self.state = State::Unlocking;
                                
                                spawn_with_result(&unlock_result, async move {
                                    sleep(Duration::from_secs(2)).await;
                                    wallet.wallet_open(wallet_secret, wallet_name, true, true).await?;
                                    // wallet.load(secret, wallet_name).await?;
                                    Ok(())
                                });

                                
                            }

                            ui.label(" ");

                            if ui
                                .add_sized(size, egui::Button::new(i18n("Select a different wallet")))
                                .clicked()
                            {
                                self.state = State::Select;
                            }
                        });
                }
                State::Unlocking => {
                    ui.heading(i18n("Unlocking"));
                    // ui.separator();
                    ui.label(" ");
                    ui.label(i18n("Unlocking wallet, please wait..."));
                    ui.label(" ");
                    ui.add_space(64.);
                    ui.add(egui::Spinner::new().size(92.));

                    if let Some(result) = unlock_result.take() {
                        match result {
                            Ok(_) => {
                                // self.state = State::Unlock;
                                core.select::<modules::AccountManager>();
                            }
                            Err(err) => {
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
