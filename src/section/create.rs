use crate::stages::*;
use crate::{imports::*, interop::spawn_with_result};
use egui::*;
use kaspa_wallet_core::runtime::{WalletCreateArgs, PrvKeyDataCreateArgs, AccountCreateArgs};
use kaspa_wallet_core::storage::interface::AccessContext;
use kaspa_wallet_core::storage::{AccountKind, AccessContextT};
use kaspa_bip32::Mnemonic;
// use workflow_core::task::spawn;

#[derive(Clone)]
pub enum State {
    Start,
    WalletName, // Wallet and Account name
    AccountName,
    PhishingHint,
    WalletSecret,
    PaymentSecret,
    CreateWallet,
    WalletError(Arc<Error>),
    PresentMnemonic(Arc<Mnemonic>),
    ConfirmMnemonic,
    Finish,
}

#[derive(Clone, Default)]
struct CreateWallet {
    wallet_title: String,
    wallet_filename : String,
    account_title: String,
    phishing_hint: String,
    wallet_secret: String,
    wallet_secret_confirm: String,
    enable_payment_secret: bool,
    payment_secret: String,
    payment_secret_confirm: String,
    mnemonic: Vec<String>,
}

pub struct Create {
    #[allow(dead_code)]
    interop: Interop,
    secret: String,
    args : CreateWallet,
    pub state: State,
    // pub message: Option<String>,
    // selected_wallet : Option<String>,
}

impl Create {
    pub fn new(interop: Interop) -> Self {
        Self {
            interop,
            secret: String::new(),
            state: State::Start,
            args : Default::default(),
            // message: None,
            // selected_wallet : None,
        }
    }

}

impl SectionT for Create {
    fn render(
        &mut self,
        wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {

            let size = egui::Vec2::new(200_f32, 40_f32);
            let wallet_exists_result = Payload::<Result<bool>>::new("wallet_exists_result");

            match self.state.clone() {
                State::Start => {
                    ui.label(" ");
                    ui.heading("Create a Wallet");
                    ui.label(" ");
                    ui.label(" ");
                    ui.label("The following will guide you through the process of creating a new wallet");
                    ui.label(" ");
                    ui.label("A wallet is stored in a file on your computer. You can create multiple wallet.");
                    ui.label(" ");

                    ui.add_space(32.);

                    egui::ScrollArea::vertical()
                        // .id_source("wallet-list")
                        
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(ui.available_height()-64.);

                        });
                    if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                        self.state = State::WalletName;
                    }
                    // ui.add_space(64.);
        
                }
                State::WalletName => {
                    ui.heading("Wallet Name");
                    ui.label(" ");
                    ui.label("Please specify the name of the wallet");
                    ui.label(" ");
                    // ui.add_space(32.);

                    egui::ScrollArea::vertical()
                        // .id_source("wallet-list")
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(ui.available_height()-64.);

                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut self.args.wallet_title)
                                    .hint_text("Wallet Name...")
                                    .vertical_align(Align::Center),
                            );

                            ui.add_space(32.);

                        });
                    if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                        self.state = State::AccountName;
                    }
        
                }
                State::AccountName => {
                    ui.heading("Default Account Name");
                    ui.label(" ");
                    ui.heading("Optional");
                    ui.label(" ");
                    ui.label("Please specify the name of the default account. The wallet will be created with a default account. Once created you will be able to create additional accounts as you need.");
                    ui.label(" ");
                    ui.label("If not specified, the account will be represented by it's numeric id.");

                    ui.add_space(32.);

                    egui::ScrollArea::vertical()
                        // .id_source("wallet-list")
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());

                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut self.args.account_title)
                                    .hint_text("Account Name...")
                                    .vertical_align(Align::Center),
                            );

                            if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                                self.state = State::PhishingHint;
                            }
                        });
        
                }
                State::PhishingHint => {
                    ui.heading("Phishing Hint");
                    ui.label(" ");
                    ui.heading("Optional");
                    ui.label(" ");
                    ui.label("\
                    \"Phishing hint\" is a secret word or a phrase that is displayed \
                    when you open your wallet. If you do not see the hint when opening \
                    your wallet, you may be accessing a fake wallet designed to steal \
                    your funds. If this occurs, stop using the wallet immediately, \
                    check the browser URL domain name and seek help on social networks \
                    (Kaspa Discord or Telegram).");
                    ui.label(" ");

                    ui.add_space(32.);

                    egui::ScrollArea::vertical()
                        // .id_source("wallet-list")
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());

                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut self.args.phishing_hint)
                                    .hint_text("Phishing hint...")
                                    .vertical_align(Align::Center),
                            );

                            if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                                self.state = State::WalletSecret;
                            }
                        });
        
                }
                State::WalletSecret => {
                    ui.heading("Wallet Encryption Password");
                    ui.label(" ");
                    ui.label("Wallet password is used to encrypt your wallet.");
                    ui.label(" ");

                    ui.add_space(32.);

                    egui::ScrollArea::vertical()
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());

                            ui.label(" ");
                            ui.label(egui::RichText::new("ENTER YOUR PASSWORD").size(12.).raised());
                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut self.args.wallet_secret)
                                    .hint_text("Wallet password...")
                                    .vertical_align(Align::Center),
                            );

                            ui.label(" ");
                            ui.label(egui::RichText::new("VERIFY YOUR PASSWORD").size(12.).raised());

                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut self.args.wallet_secret_confirm)
                                    .hint_text("Verify password...")
                                    .vertical_align(Align::Center),
                            );

                            if self.args.wallet_secret_confirm.len() > 0 && self.args.wallet_secret != self.args.wallet_secret_confirm {
                                ui.label(" ");
                                ui.label(egui::RichText::new("Passwords do not match").color(egui::Color32::from_rgb(255, 120, 120)));
                                ui.label(" ");
                            } else {
                                ui.label(" ");
                                ui.label(" ");
                                ui.label(" ");
                            }

                            let ok = self.args.wallet_secret == self.args.wallet_secret_confirm && self.args.wallet_secret.len() > 0;
                            if ui.add_enabled(ok, egui::Button::new("Continue").min_size(size)).clicked() {
                                self.state = State::PaymentSecret;
                            }
                            // if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                            //     self.state = State::AccountName;
                            // }
                        });
        
                }
                State::PaymentSecret => {
                    ui.heading("Payment & Recovery Password");
                    ui.label(" ");
                    ui.heading("Optional");
                    ui.label(" ");
                    
                    egui::ScrollArea::vertical()
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());

                            ui.label("The optional payment & recovery password, if provided, will be required to \
                            send payments. This password will also be required when recovering your wallet \
                            in addition to your private key or mnemonic. If you loose this password, you will not \
                            be able to use mnemonic to recover your wallet!");
                            ui.label(" ");
        
                            ui.add_space(32.);

                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut self.args.payment_secret)
                                    .hint_text("Payment password...")
                                    .vertical_align(Align::Center),
                            );

                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut self.args.payment_secret_confirm)
                                    .hint_text("Payment password...")
                                    .vertical_align(Align::Center),
                            );

                            let ok = self.args.wallet_secret == self.args.wallet_secret_confirm;
                            if ui.add_enabled(ok, egui::Button::new("Continue").min_size(size)).clicked() {
                                self.state = State::CreateWallet;
                            }
                            // if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                            //     self.state = State::AccountName;
                            // }
                        });
        
                }
                State::CreateWallet => {

                    ui.heading("Creating Wallet");
                    ui.label(" ");
                    ui.heading("Please wait...");
                    ui.label(" ");
                    ui.label(" ");
                    ui.add_space(64.);
                    ui.add(egui::Spinner::new().size(92.));

                    let wallet_create_result = Payload::<Result<Arc<Mnemonic>>>::new("wallet_create_result");
                    if !wallet_create_result.is_pending() {

                        let wallet = self.interop.wallet();
                        spawn_with_result(&wallet_create_result, async move {
                            // suspend commits for multiple operations
                            // wallet.store().batch().await?;

                            // let account_kind = AccountKind::Bip32;
                            // let wallet_args = WalletCreateArgs::new(name.map(String::from), hint, wallet_secret.clone(), true);
                            // let prv_key_data_args = PrvKeyDataCreateArgs::new(None, wallet_secret.clone(), payment_secret.clone());
                            // let account_args = AccountCreateArgs::new(account_name, account_title, account_kind, wallet_secret.clone(), payment_secret);
                            // let descriptor = wallet.create_wallet(wallet_args).await?;
                            // let (prv_key_data_id, mnemonic) = wallet.create_prv_key_data(prv_key_data_args).await?;
                            // let account = wallet.create_bip32_account(prv_key_data_id, account_args).await?;

                            // // flush data to storage
                            // let access_ctx: Arc<dyn AccessContextT> = Arc::new(AccessContext::new(wallet_secret));
                            // wallet.store().flush(&access_ctx).await?;
                            Ok(Arc::new(Mnemonic::create_random()?))
                        });
                    }

                    if let Some(result) = wallet_create_result.take() {
                        match result {
                            Ok(mnemonic) => {
                                println!("Wallet created successfully");
                                self.state = State::PresentMnemonic(mnemonic);
                            }
                            Err(err) => {
                                println!("Wallet creation error: {}", err);
                                self.state = State::WalletError(Arc::new(err));
                            }
                        }
                    }

                }

                State::WalletError(err) => {
                    ui.heading("Error");
                    ui.label(" ");
                    ui.label(" ");
                    ui.label(egui::RichText::new("Error creating a wallet").color(egui::Color32::from_rgb(255, 120, 120)));
                    ui.label(egui::RichText::new(err.to_string()).color(egui::Color32::from_rgb(255, 120, 120)));

                    ui.label(" ");

                    if ui.add_sized(size, egui::Button::new("Restart")).clicked() {
                        self.state = State::Start;
                    }
                }

                State::PresentMnemonic(mnemonic) => {
                    ui.heading("Private Key Mnemonic");
                    ui.label(" ");

                    egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        ui.label("Your mnemonic phrase allows your to re-create your private key. \
                        The person who has access to this mnemonic will have full control of \
                        the Kaspa stored in it. Keep your mnemonic safe. Write it down and \
                        store it in a safe, preferably in a fire-resistant location. Do not \
                        store your mnemonic on this computer or a mobile device. This wallet \
                        will never ask you for this mnemonic phrase unless you manually \
                        initiate a private key recovery.");
                        ui.label(" ");
                        ui.label("Never share your mnemonic with anyone!");
                        ui.label(" ");
                        ui.label("Your default account private key mnemonic is:");
                        ui.separator();
                        ui.label(egui::RichText::new(mnemonic.phrase()).color(egui::Color32::WHITE));

                        ui.add(TextEdit::multiline(&mut self.args.payment_secret_confirm)
                        // .hint_text("Payment password...")
                        
                        .vertical_align(Align::Center));



                        ui.separator();
                        ui.label(" ");

                        if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                            self.state = State::ConfirmMnemonic;
                        }

                        /*
                        if ui
                            .add(
                                egui::Label::new("Label Text")
                                    .sense(egui::Sense::click()),
                            )
                            .on_hover_text("Click to copy 'Label Text'")
                            .clicked()
                        {
                            ui.output_mut(|po| {
                                po.copied_text = String::from("Label Text");
                            });
                        }
                         */
                    });
                }

                State::ConfirmMnemonic => {
                    ui.heading("Confirm Mnemonic");
                    ui.label(" ");
                    ui.label(" ");
                    ui.separator();
                    // ui.label(egui::RichText::new(mnemonic.phrase()).color(egui::Color32::WHITE));
                    ui.separator();
                    ui.label(" ");

                    if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                        self.state = State::ConfirmMnemonic;
                    }
                }

                State::Finish => {
                    ui.heading("Wallet Created");
                    ui.label(" ");
                    ui.label("Your wallet has been created and is ready to use.");
                    ui.label(" ");

                    if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                        self.state = State::Start;
                        wallet.select::<section::Overview>();
                    }
                }

            }

        });
    }
}
