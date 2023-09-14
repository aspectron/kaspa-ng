#![allow(unused_imports)]

use crate::imports::*;
use kaspa_bip32::Mnemonic;
use kaspa_wallet_core::runtime::{AccountCreateArgs, PrvKeyDataCreateArgs, WalletCreateArgs};
use kaspa_wallet_core::storage::interface::AccessContext;
use kaspa_wallet_core::storage::{AccessContextT, AccountKind};

#[derive(Clone)]
pub enum State {
    Start,
    WalletName,
    AccountName,
    PhishingHint,
    WalletSecret,
    PaymentSecret,
    CreateWallet,
    WalletError(Arc<Error>),
    PresentMnemonic(Arc<Mnemonic>),
    ConfirmMnemonic(Arc<Mnemonic>),
    Finish,
}

#[derive(Clone, Default)]
struct Context {
    wallet_title: String,
    // TODO generate wallet filename
    wallet_filename: String,
    account_title: String,
    account_name: String,
    enable_phishing_hint: bool,
    phishing_hint: String,
    wallet_secret: String,
    wallet_secret_confirm: String,
    // TODO add payment secret checkbox
    enable_payment_secret: bool,
    payment_secret: String,
    payment_secret_confirm: String,
    // mnemonic: Vec<String>,
}

pub struct CreateWallet {
    #[allow(dead_code)]
    interop: Interop,
    // secret: String,
    args: Context,
    pub state: State,
}

impl CreateWallet {
    pub fn new(interop: Interop) -> Self {
        Self {
            interop,
            // secret: String::new(),
            state: State::Start,
            args: Default::default(),
        }
    }
}

impl SectionT for CreateWallet {
    fn render(
        &mut self,
        wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {

            let size = egui::Vec2::new(200_f32, 40_f32);

            match self.state.clone() {
                State::Start => {

                    Panel::new(self)
                        .with_caption("Create Wallet")
                        .with_close_enabled(false, |_|{
                        })
                        .with_header(|_ctx,ui| {
                            ui.add_space(64.);
                            ui.label("The following will guide you through the process of creating a new wallet");
                            ui.label(" ");
                            ui.label("A wallet is stored in a file on your computer. You can create multiple wallet.");
                        })
                        .with_footer(|this,ui| {
                            // if ui.add_sized(theme().large_button_size, egui::Button::new("Continue")).clicked() {
                            let size = theme().large_button_size;
                            if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                                this.state = State::WalletName;
                            }
                        })
                        .render(ui);
                }
                State::WalletName => {

                    // TODO - check if wallet exists
                    let _wallet_exists_result = Payload::<Result<bool>>::new("wallet_exists_result");

                    Panel::new(self)
                    .with_caption("Wallet Name")
                    .with_back(|this| {
                        this.state = State::Start;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.add_space(64.);
                        ui.label("Please specify the name of the wallet");
                    })
                    .with_body(|this,ui| {
                        ui.add_sized(
                            size,
                            TextEdit::singleline(&mut this.args.wallet_title)
                                .hint_text("Wallet Name...")
                                .vertical_align(Align::Center),
                        );
                    })
                    .with_footer(|this,ui| {
                        let size = theme().large_button_size;
                        if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                            this.state = State::AccountName;
                        }
                    })
                    .render(ui);
                }
                State::AccountName => {
                    Panel::new(self)
                        .with_caption("Default Account Name")
                        .with_back(|this| {
                            this.state = State::WalletName;
                        })
                        .with_close_enabled(false, |_|{
                        })
                        .with_header(|_ctx,ui| {
                            ui.heading("Optional");
                            ui.label(" ");
                            ui.label("Please specify the name of the default account. The wallet will be created with a default account. Once created you will be able to create additional accounts as you need.");
                            ui.label(" ");
                            ui.label("If not specified, the account will be represented by it's numeric id.");
                        })
                        .with_body(|this,ui| {
                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut this.args.account_title)
                                    .hint_text("Account Name...")
                                    .vertical_align(Align::Center),
                            );

                        })
                        .with_footer(|this,ui| {
                            let size = theme().large_button_size;
                            if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                                this.state = State::PhishingHint;
                            }
                        })
                        .render(ui);
                }
                State::PhishingHint => {

                    Panel::new(self)
                        .with_caption("Phishing Hint")
                        .with_back(|this| {
                            this.state = State::AccountName;
                        })
                        .with_close_enabled(false, |_|{
                        })
                        .with_header(|_ctx,ui| {
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
                        })
                        .with_body(|this,ui| {
                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut this.args.phishing_hint)
                                    .hint_text("Phishing hint...")
                                    .vertical_align(Align::Center),
                            );

                        })
                        .with_footer(|this,ui| {
                            let size = theme().large_button_size;
                            if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                                this.state = State::WalletSecret;
                            }
                        })
                        .render(ui);
                }
                State::WalletSecret => {

                    Panel::new(self)
                        .with_caption("Wallet Encryption Password")
                        .with_back(|this| {
                            this.state = State::AccountName;
                        })
                        .with_close_enabled(false, |_|{
                        })
                        .with_header(|_ctx,ui| {
                            ui.label(" ");
                            ui.label("Wallet password is used to encrypt your wallet data.");
                            ui.label(" ");
                        })
                        .with_body(|this,ui| {
                            ui.label(" ");
                            ui.label(egui::RichText::new("ENTER YOUR WALLET PASSWORD").size(12.).raised());
                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut this.args.wallet_secret)
                                    .hint_text("Wallet password...")
                                    .vertical_align(Align::Center),
                            );

                            ui.label(" ");
                            ui.label(egui::RichText::new("VERIFY YOUR WALLET PASSWORD").size(12.).raised());

                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut this.args.wallet_secret_confirm)
                                    .hint_text("Verify password...")
                                    .vertical_align(Align::Center),
                            );

                            if this.args.wallet_secret_confirm.is_not_empty() && this.args.wallet_secret != this.args.wallet_secret_confirm {
                                ui.label(" ");
                                ui.label(egui::RichText::new("Passwords do not match").color(egui::Color32::from_rgb(255, 120, 120)));
                                ui.label(" ");
                            } else {
                                ui.label(" ");
                            }
                        })
                        .with_footer(|this,ui| {
                            let size = theme().large_button_size;
                            let ok = this.args.wallet_secret == this.args.wallet_secret_confirm && this.args.wallet_secret.is_not_empty();
                            if ui.add_enabled(ok, egui::Button::new("Continue").min_size(size)).clicked() {
                                this.state = State::PaymentSecret;
                            }
                        })
                        .render(ui);
                }
                State::PaymentSecret => {

                    Panel::new(self)
                        .with_caption("Payment & Recovery Password")
                        .with_back(|this| {
                            this.state = State::AccountName;
                        })
                        .with_close_enabled(false, |_|{
                        })
                        .with_header(|_ctx,ui| {
                            ui.heading("Optional");
                            ui.label(" ");
                            ui.label("The optional payment & recovery password, if provided, will be required to \
                                send payments. This password will also be required when recovering your wallet \
                                in addition to your private key or mnemonic. If you loose this password, you will not \
                                be able to use mnemonic to recover your wallet!");
                        })
                        .with_body(|this,ui| {
                            ui.label(egui::RichText::new("ENTER YOUR PAYMENT PASSWORD").size(12.).raised());
                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut this.args.payment_secret)
                                    .hint_text("Payment password...")
                                    .vertical_align(Align::Center),
                            );

                            ui.label(" ");
                            ui.label(egui::RichText::new("VERIFY YOUR PAYMENT PASSWORD").size(12.).raised());

                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut this.args.payment_secret_confirm)
                                    .hint_text("Payment password...")
                                    .vertical_align(Align::Center),
                            );

                            if this.args.wallet_secret_confirm.is_not_empty() && this.args.payment_secret != this.args.payment_secret_confirm {
                                ui.label(" ");
                                ui.label(egui::RichText::new("Passwords do not match").color(egui::Color32::from_rgb(255, 120, 120)));
                                ui.label(" ");
                            } else {
                                ui.label(" ");
                            }
                        })
                        .with_footer(|this,ui| {
                            let size = theme().large_button_size;
                            let ok = this.args.payment_secret == this.args.payment_secret_confirm;// && this.args.wallet_secret.len() > 0;
                            if ui.add_enabled(ok, egui::Button::new("Continue").min_size(size)).clicked() {
                                this.state = State::CreateWallet;
                            }
                        })
                        .render(ui);
                }
                State::CreateWallet => {

                    Panel::new(self)
                    .with_caption("Creating Wallet")
                    .with_header(|_, ui|{
                        ui.label(" ");
                        ui.label("Please wait...");
                        ui.label(" ");
                        ui.label(" ");
                        ui.add_space(64.);
                        ui.add(egui::Spinner::new().size(92.));
                    })
                    .render(ui);

                    let args = self.args.clone();
                    let wallet_create_result = Payload::<Result<(Arc<Mnemonic>,Arc<dyn runtime::Account>)>>::new("wallet_create_result");
                    if !wallet_create_result.is_pending() {

                        // TODO CREATE WALLET !
                        let wallet = self.interop.wallet().clone();
                        spawn_with_result(&wallet_create_result, async move {

                            if args.enable_payment_secret && args.payment_secret.is_empty() {
                                return Err(Error::custom("Payment secret is empty"));
                            }

                            if args.enable_phishing_hint && args.phishing_hint.is_empty() {
                                return Err(Error::custom("Phishing hint is empty"));
                            }

                            let wallet_secret = Secret::from(args.wallet_secret);
                            let payment_secret = args.enable_payment_secret.then_some(Secret::from(args.payment_secret));

                            // suspend commits for multiple operations
                            wallet.store().batch().await?;

                            let account_kind = AccountKind::Bip32;
                            let wallet_args = WalletCreateArgs::new(
                                args.wallet_title.is_not_empty().then_some(args.wallet_title),
                                args.wallet_filename.is_not_empty().then_some(args.wallet_filename),
                                args.enable_phishing_hint.then_some(args.phishing_hint.into()),
                                wallet_secret.clone(),
                                false
                            );
                            let prv_key_data_args = PrvKeyDataCreateArgs::new(
                                None,
                                wallet_secret.clone(),
                                payment_secret.clone()
                            );
                            let account_args = AccountCreateArgs::new(
                                args.account_name.is_not_empty().then_some(args.account_name),
                                args.account_title.is_not_empty().then_some(args.account_title),
                                account_kind,
                                wallet_secret.clone(),
                                payment_secret.clone(),
                            );
                            let _descriptor = wallet.create_wallet(wallet_args).await?;
                            let (prv_key_data_id, mnemonic) = wallet.create_prv_key_data(prv_key_data_args).await?;
                            let account = wallet.create_bip32_account(prv_key_data_id, account_args).await?;

                            // flush data to storage
                            let access_ctx: Arc<dyn AccessContextT> = Arc::new(AccessContext::new(wallet_secret));
                            wallet.store().flush(&access_ctx).await?;

                            Ok((Arc::new(mnemonic), account))
                        });
                    }

                    if let Some(result) = wallet_create_result.take() {
                        match result {
                            Ok((mnemonic,account)) => {
                                println!("Wallet created successfully");
                                self.state = State::PresentMnemonic(mnemonic);
                                wallet.get_mut::<section::Account>().select(Some(account));
                            }
                            Err(err) => {
                                println!("Wallet creation error: {}", err);
                                self.state = State::WalletError(Arc::new(err));
                            }
                        }
                    }

                }

                State::WalletError(err) => {

                    Panel::new(self)
                    .with_caption("Error")
                    .with_header(move |this,ui| {
                        ui.label(" ");
                        ui.label(" ");
                        ui.label(egui::RichText::new("Error creating a wallet").color(egui::Color32::from_rgb(255, 120, 120)));
                        ui.label(egui::RichText::new(err.to_string()).color(egui::Color32::from_rgb(255, 120, 120)));

                        if ui.add_sized(size, egui::Button::new("Restart")).clicked() {
                            this.state = State::Start;
                        }
                    })
                    .render(ui);
                }

                State::PresentMnemonic(mnemonic) => {
                    let mut phrase = mnemonic.phrase().to_string();

                    Panel::new(self)
                        .with_caption("Private Key Mnemonic")
                        .with_body(|_this,ui| {
                            ui.label(RichText::new("Your mnemonic phrase allows your to re-create your private key. \
                                The person who has access to this mnemonic will have full control of \
                                the Kaspa stored in it. Keep your mnemonic safe. Write it down and \
                                store it in a safe, preferably in a fire-resistant location. Do not \
                                store your mnemonic on this computer or a mobile device. This wallet \
                                will never ask you for this mnemonic phrase unless you manually \
                                initiate a private key recovery.").size(14.));
                            ui.label(" ");
                            ui.label(RichText::new("Never share your mnemonic with anyone!").color(Color32::RED));
                            ui.label(" ");
                            ui.label("Your default account private key mnemonic is:");
                            ui.label(" ");
                            ui.separator();
                            ui.label(" ");

                            // let phrase = mnemonic.phrase();
                            let words = phrase.split(' ').collect::<Vec<&str>>();
                            let chunks = words.chunks(6).collect::<Vec<&[&str]>>();
                            // let lines = chunks.iter().map(|chunk| chunk.join(" ")).collect::<Vec<String>>();
                            // let text = lines.join("\n");
                            for chunk in chunks {
                            // for col in chunks.len() {
                                ui.horizontal(|ui| {
                                    ui.columns(6, |cols| {

                                        for col in 0..chunk.len() {
                                            cols[col].label(egui::RichText::new(chunk[col]).family(FontFamily::Monospace).size(14.).color(egui::Color32::WHITE));
                                        }
                                    })
                            //         for word in chunk {
                            //             ui.label(egui::RichText::new(*word).family(FontFamily::Monospace).color(egui::Color32::WHITE));
                            //             ui.add_space(8.);
                            //         }
                                });
                            }

                            phrase.zeroize();

                            // ui.label(egui::RichText::new(text).family(FontFamily::Monospace).color(egui::Color32::WHITE));
                            // ui.label(egui::RichText::new(mnemonic.phrase()).family(FontFamily::Monospace).color(egui::Color32::WHITE));

                            // ui.add(TextEdit::multiline(&mut this.args.payment_secret_confirm)
                            // .vertical_align(Align::Center));
                    })
                    .with_footer(|this,ui| {
                        if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                            this.state = State::ConfirmMnemonic(mnemonic);
                        }
                    })
                    .render(ui);

                }

                State::ConfirmMnemonic(mnemonic) => {
                    Panel::new(self)
                        .with_caption("Confirm Mnemonic")
                        .with_back(|this|{
                            this.state = State::PresentMnemonic(mnemonic);
                        })
                        .with_header(|_this,ui| {
                            ui.label("Please validate your mnemonic");
                        })
                        .with_footer(move |this,ui| {
                            if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                                this.state = State::Finish;
                            }
                        })
                        .render(ui);
                }

                State::Finish => {

                    Panel::new(self)
                        .with_caption("Wallet Created")
                        .with_body(|_this,ui| {
                            ui.label(" ");
                            ui.label("Your wallet has been created and is ready to use.");
                            ui.label(" ");
                        })
                        .with_footer(move |this,ui| {
                            if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                                this.state = State::Start;
                                wallet.select::<section::Account>();
                            }
                        })
                        .render(ui);
                }

            }

        });
    }
}
