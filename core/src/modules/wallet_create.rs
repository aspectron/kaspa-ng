
use crate::imports::*;
use kaspa_wallet_core::runtime::{AccountCreateArgs, PrvKeyDataCreateArgs, WalletCreateArgs};
use slug::slugify;
use kaspa_bip32::{WordCount, Mnemonic, Language};
use crate::utils::{secret_score, secret_score_to_text};

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
enum Focus {
    #[default]
    None,
    WalletName,
    AccountName,
    PhishingHint,
    WalletSecret,
    WalletSecretConfirm,
    PaymentSecret,
    PaymentSecretConfirm,
}

#[derive(Clone)]
pub enum State {
    Start,
    KeySelection,
    ImportSelection,
    WalletName,
    AccountName,
    PhishingHint,
    WalletSecret,
    PaymentSecret,
    CreateWalletConfirm,
    CreateWallet,
    WalletError(Arc<Error>),
    PresentMnemonic(Arc<String>),
    ConfirmMnemonic(Arc<String>),
    Finish,
}

enum PrivateKeyImportKind {

}


#[derive(Clone, Default)]
struct Context {
    word_count : WordCount,
    // custom_wallet_filename : bool,
    wallet_name: String,
    // TODO generate wallet filename
    wallet_filename: String,
    // account_title: String,
    account_name: String,
    enable_phishing_hint: bool,
    phishing_hint: String,
    wallet_secret: String,
    wallet_secret_confirm: String,
    wallet_secret_show: bool,
    wallet_secret_score: Option<f64>,
    // TODO add payment secret checkbox
    enable_payment_secret: bool,
    payment_secret: String,
    payment_secret_confirm: String,
    payment_secret_show : bool,
    payment_secret_score: Option<f64>,
    mnemonic_presenter_context : MnemonicPresenterContext,
    
    // ---
    import_private_key : bool,
    import_private_key_mnemonic : String,
    import_legacy : bool,
    import_advanced : bool,
    
    // mnemonic: Vec<String>,
}

impl Zeroize for Context {
    fn zeroize(&mut self) {
        self.wallet_name.zeroize();
        self.wallet_filename.zeroize();
        self.account_name.zeroize();
        self.phishing_hint.zeroize();
        self.wallet_secret.zeroize();
        self.wallet_secret_confirm.zeroize();
        self.payment_secret.zeroize();
        self.payment_secret_confirm.zeroize();
        // self.mnemonic_presenter_context.zeroize();
    }
}

pub struct WalletCreate {
    #[allow(dead_code)]
    runtime: Runtime,
    context: Context,
    pub state: State,
    pub origin: Option<TypeId>,
    focus : FocusManager<Focus>,
}

impl WalletCreate {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            state: State::Start,
            context: Default::default(),
            origin: None,
            focus: FocusManager::default(),
        }
    }
}

impl ModuleT for WalletCreate {
    fn modal(&self) -> bool { true }

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Mobile
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        let editor_size = egui::Vec2::new(200_f32, 40_f32);

        match self.state.clone() {
            State::Start => {
                // let has_origin = self.origin.is_some();

                Panel::new(self)
                    .with_caption("Create Wallet")
                    .with_back_enabled(core.has_stack(), |_|{
                        // wallet.select_with_type_id(this.origin.take().unwrap())
                        core.back()
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.add_space(64.);
                        ui.label("The following will guide you through the process of creating a new wallet");
                        ui.label(" ");
                        ui.label("A wallet is stored in a file on your computer.");
                        ui.label("You can create multiple wallets, but only one can be loaded at a time.");
                    })
                    // .with_footer(|this,ui| {
                    //     // if ui.add_sized(theme().large_button_size, egui::Button::new("Continue")).clicked() {
                    //     let size = theme().large_button_size;
                    //     if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                    //         this.state = State::WalletName;
                    //     }
                    // })
                    .with_handler(|this| {
                        this.state = State::KeySelection;
                    })
                    .render(ui);
            }
            State::KeySelection => {

                // TODO - check if wallet exists
                // let _wallet_exists_result = Payload::<Result<bool>>::new("wallet_exists_result");
                let mut submit = false;
                let mut import = false;

                Panel::new(self)
                    .with_caption("Select Private Key Type")
                    .with_back(|this| {
                        this.state = State::Start;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.add_space(64.);
                        ui.label("Please specify the private key type\nfor the new wallet");
                    })
                    .with_body(|this,ui| {

                        if ui.large_button("12 word mnemonic").clicked() {
                            this.context.word_count = WordCount::Words12;
                            submit = true;
                        }
                        ui.label("");
                        if ui.large_button("24 word mnemonic").clicked() {
                            this.context.word_count = WordCount::Words24;
                            submit = true;
                        }
                        // ui.label("");
                        // if ui.large_button_enabled(false,"MultiSig account").clicked() {
                        // }
                        ui.label("");
                        ui.separator();
                        ui.label("");
                        ui.label("Other operations");
                        ui.label("");

                        if ui.large_button("Import existing").clicked() {
                            // this.context.word_count = WordCount::Words24;
                            import = true;
                        }
                        ui.label("");

                    })
                    .with_footer(|_this,_ui| {
                    })
                    .render(ui);

                    if import {
                        self.state = State::ImportSelection;
                        self.focus.clear();

                    } else if submit {
                        self.state = State::WalletName;
                        self.focus.next(Focus::WalletName);
                    }

            }

            State::ImportSelection => {

                let mut submit = false;
                Panel::new(self)
                    .with_caption("Import Existing Private Key")
                    .with_back(|this| {
                        this.state = State::KeySelection;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.add_space(64.);
                        ui.label("Please select the private key type you would like to import in the new wallet");
                        // ui.label("");
                    })
                    .with_body(|this,ui| {
                        // ui.label("(You can import additional private keys later, once the wallet has been created)");

                        if ui.large_button("12 word mnemonic").clicked() {
                            this.context.word_count = WordCount::Words12;
                            submit = true;
                        }

                        ui.label("");
                        ui.checkbox(&mut this.context.import_legacy, "I have a legacy account");
                        ui.label("Select this option if your wallet was create\nvia KDX or kaspanet.io web wallet");

                        if !this.context.import_legacy {
                            ui.label("");
                            if ui.large_button("24 word mnemonic").clicked() {
                                this.context.word_count = WordCount::Words24;
                                submit = true;
                            }
                            // ui.label("");
                            // if ui.large_button("MultiSig account").clicked() {
                            // }
                        }

                        ui.label("");
                        ui.checkbox(&mut this.context.import_advanced, "Advanced Options");
                        if this.context.import_advanced {
                            ui.label("");
                            if ui.large_button("secp256k1 keypair").clicked() {
                                this.context.word_count = WordCount::Words12;
                                submit = true;
                            }
                        }

                    })
                    .with_footer(|_this,_ui| {
                    })
                    .render(ui);

                    if submit {
                        self.state = State::WalletName;
                        self.focus.next(Focus::WalletName);
                    }
            }


            State::WalletName => {

                // TODO - check if wallet exists
                let _wallet_exists_result = Payload::<Result<bool>>::new("wallet_exists_result");

                Panel::new(self)
                    .with_caption("Wallet Name")
                    .with_back(|this| {
                        this.state = State::KeySelection;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.add_space(64.);
                        ui.label("Please specify the name of the new wallet");
                    })
                    .with_body(|this,ui| {
                        // let response = 

                        let mut change = false;
                        TextEditor::new(
                            &mut this.context.wallet_name,
                            &mut this.focus,
                            Focus::WalletName,
                            |ui, text| {
                                // ui.add_space(8.);
                                ui.label(RichText::new("Enter wallet name").size(12.).raised());
                                ui.add_sized(editor_size, TextEdit::singleline(text)
                                    .vertical_align(Align::Center))
                            },
                        )
                        .change(|_text| {
                            change = true;
                        })
                        .submit(|text,focus| {
                            if text.is_not_empty() {
                                this.state = State::AccountName;
                                focus.next(Focus::AccountName);
                            }
                        })
                        .build(ui);

                        if change {
                            this.context.wallet_filename = slugify(&this.context.wallet_name);
                        }

                        if this.context.wallet_filename.is_not_empty() {
                            ui.label(" ");
                            ui.label(format!("Filename: {}", this.context.wallet_filename));
                            ui.label(" ");
                        }

                    })
                    .with_footer(|this,ui| {
                        if ui.large_button_enabled(this.context.wallet_name.is_not_empty(), "Continue").clicked() {
                            this.state = State::AccountName;
                            this.focus.next(Focus::AccountName);
                        }

                    })
                    .render(ui);
            }
            State::AccountName => {
                Panel::new(self)
                    .with_caption("Default Account Name")
                    .with_back(|this| {
                        this.state = State::WalletName;
                        this.focus.next(Focus::WalletName);
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.heading("Optional");
                        ui.label(" ");
                        ui.label("Please specify the name of the default account. The wallet will be created with a default account. Once created, you will be able to create additional accounts as you need.");
                        ui.label(" ");
                        ui.label("If not specified, the account will be represented by the numeric id.");
                    })
                    .with_body(|this,ui| {

                        TextEditor::new(
                            &mut this.context.account_name,
                            &mut this.focus,
                            Focus::AccountName,
                            |ui, text| {
                                // ui.add_space(8.);
                                ui.label(RichText::new("Enter account name").size(12.).raised());
                                ui.add_sized(editor_size, TextEdit::singleline(text)
                                    .vertical_align(Align::Center))
                            },
                        )
                        .submit(|_text,focus| {
                            this.state = State::PhishingHint;
                            focus.next(Focus::PhishingHint);
                        })
                        .build(ui);

                    })
                    .with_footer(|this,ui| {
                        // let size = theme().large_button_size;
                        let text = if this.context.account_name.is_not_empty() { "Continue" } else { "Skip" };
                        if ui.large_button(i18n(text)).clicked() {
                            this.state = State::PhishingHint;
                            this.focus.next(Focus::PhishingHint);
                        }
                        // if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                        //     this.state = State::PhishingHint;
                        // }
                    })
                    .render(ui);
            }
            State::PhishingHint => {

                Panel::new(self)
                    .with_caption("Phishing Hint")
                    .with_back(|this| {
                        this.state = State::AccountName;
                        this.focus.next(Focus::AccountName);
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
                        TextEditor::new(
                            &mut this.context.phishing_hint,
                            &mut this.focus,
                            Focus::PhishingHint,
                            |ui, text| {
                                ui.label(RichText::new("Enter phishing hint").size(12.).raised());
                                ui.add_sized(editor_size, TextEdit::singleline(text)
                                    .vertical_align(Align::Center))
                            },
                        )
                        .submit(|_text,focus| {
                            this.state = State::WalletSecret;
                            focus.next(Focus::WalletSecret);
                        })
                        .build(ui);

                    })
                    .with_footer(|this,ui| {
                        if ui.large_button("Continue").clicked() {
                            this.state = State::WalletSecret;
                            this.focus.next(Focus::WalletSecret);
                        }
                    })
                    // .with_handler(|this| {
                    //     this.state = State::WalletSecret;
                    //     this.focus = Focus::WalletSecret;
                    // })
                    .render(ui);
            }
            State::WalletSecret => {

                Panel::new(self)
                    .with_caption("Wallet Encryption Password")
                    .with_back(|this| {
                        this.state = State::PhishingHint;
                        this.focus.next(Focus::PhishingHint);
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.label(" ");
                        ui.label("Wallet password is used to encrypt your wallet data.");
                        ui.label(" ");
                    })
                    .with_body(|this,ui| {

                        let mut submit = false;
                        let mut change = false;

                        TextEditor::new(
                            &mut this.context.wallet_secret,
                            &mut this.focus,
                            Focus::WalletSecret,
                            |ui, text| {
                                ui.label(RichText::new("Enter wallet password").size(12.).raised());
                                ui.add_sized(editor_size, TextEdit::singleline(text)
                                    .password(!this.context.wallet_secret_show)
                                    .vertical_align(Align::Center))
                            },
                        )
                        .change(|_|{
                            change = true;
                        })
                        .submit(|_text,focus| {
                            focus.next(Focus::WalletSecretConfirm);
                            if this.context.wallet_secret_confirm.is_not_empty() {
                                submit = true;
                            }
                        })
                        .build(ui);

                        ui.label("");

                        TextEditor::new(
                            &mut this.context.wallet_secret_confirm,
                            &mut this.focus,
                            Focus::WalletSecretConfirm,
                            |ui, text| {
                                ui.label(RichText::new("Confirm wallet password").size(12.).raised());
                                ui.add_sized(editor_size, TextEdit::singleline(text)
                                    .password(!this.context.wallet_secret_show)
                                    .vertical_align(Align::Center))
                            },
                        )
                        .submit(|_text,focus| {
                            if this.context.wallet_secret.is_empty() {
                                focus.next(Focus::WalletSecret);
                            } else {
                                submit = true;
                            }
                        })
                        .build(ui);

                        ui.label("");
                        ui.checkbox(&mut this.context.wallet_secret_show, "Show password");
                        ui.label("");


                        if change {
                            let wallet_secret = this
                                .context
                                .wallet_secret
                                .is_not_empty()
                                .then_some(this.context.wallet_secret.clone())
                                .or(this.context
                                    .wallet_secret_confirm
                                    .is_not_empty()
                                    .then_some(this.context.wallet_secret_confirm.clone())
                                );
                            this.context.wallet_secret_score = wallet_secret.map(secret_score); //Some(password_score(&this.context.wallet_secret));
                        }

                        if let Some(score) = this.context.wallet_secret_score {
                            ui.label("");
                            ui.label(format!("Secret score: {}",secret_score_to_text(score)));
                            if score < 80.0 {
                                ui.label("");
                                ui.label(RichText::new(i18n("Secret is too weak")).color(error_color()));
                                if !core.settings.developer.disable_password_restrictions() {
                                    submit = false;
                                    ui.label(RichText::new(i18n("Please create a stronger password")).color(error_color()));
                                }
                            }
                        }
                        ui.label("");

                        if this.context.wallet_secret_confirm.is_not_empty() && this.context.wallet_secret != this.context.wallet_secret_confirm {
                            ui.label(" ");
                            ui.label(RichText::new(i18n("Passwords do not match")).color(error_color()));
                            ui.label(" ");
                            submit = false;
                        } else {
                            ui.label(" ");
                        }

                        if submit {
                            this.state = State::PaymentSecret;
                            this.focus.next(Focus::PaymentSecret);
                        }
                    })
                    .with_footer(|this,ui| {
                        let is_weak = !core.settings.developer.disable_password_restrictions() && this.context.wallet_secret_score.unwrap_or_default() < 80.0;
                        let enabled = this.context.wallet_secret == this.context.wallet_secret_confirm && this.context.wallet_secret.is_not_empty();
                        if ui.large_button_enabled(enabled && !is_weak, "Continue").clicked() {
                            this.state = State::PaymentSecret;
                            this.focus.next(Focus::PaymentSecret);
                        }
                    })
                    .render(ui);
            }
            State::PaymentSecret => {


                Panel::new(self)
                    .with_caption("Payment & Recovery Password")
                    .with_back(|this| {
                        this.state = State::WalletSecret;
                        this.focus.next(Focus::WalletSecret);
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.heading("Optional");
                        ui.label(" ");
                        ui.label("The optional payment & mnemonic recovery passphrase, known as BIP39 passphrase, if provided, will be required to \
                            send payments. This passphrase will also be required when recovering your wallet in addition to your mnemonic.\
                            If you loose or forget this passphrase, you will not \
                            be able to use mnemonic to recover your wallet!");
                    })
                    .with_body(|this,ui| {
                        let mut submit = false;
                        let mut change = false;
    
    
                        ui.checkbox(&mut this.context.enable_payment_secret, i18n("Enable optional BIP39 passphrase"));

                        if this.context.enable_payment_secret {
                            
                            ui.label("");

                            TextEditor::new(
                                &mut this.context.payment_secret,
                                &mut this.focus,
                                Focus::PaymentSecret,
                                |ui, text| {
                                    // ui.add_space(8.);
                                    ui.label(RichText::new("Enter BIP39 passphrase").size(12.).raised());
                                    ui.add_sized(editor_size, TextEdit::singleline(text)
                                        .password(this.context.payment_secret_show)
                                        .vertical_align(Align::Center))
                                },
                            )
                            .change(|_|{
                                change = true;
                            })
                            .submit(|text,focus| {
                                if text.is_not_empty() {
                                    focus.next(Focus::PaymentSecretConfirm);
                                } else {
                                    submit = true;
                                }
                            })
                            .build(ui);

                            ui.label("");
                            TextEditor::new(
                                &mut this.context.payment_secret_confirm,
                                &mut this.focus,
                                Focus::PaymentSecretConfirm,
                                |ui, text| {
                                    ui.label(RichText::new("Confirm BIP39 passphrase").size(12.).raised());
                                    ui.add_sized(editor_size, TextEdit::singleline(text)
                                        .password(this.context.payment_secret_show)
                                        .vertical_align(Align::Center))
                                },
                            )
                            .submit(|_text,_focus| {
                                submit = true;
                            })
                            .build(ui);

                            ui.label("");
                            ui.checkbox(&mut this.context.payment_secret_show, "Show passphrase");
                            ui.label("");

                            if change {
                                let payment_secret = this
                                    .context
                                    .payment_secret
                                    .is_not_empty()
                                    .then_some(this.context.payment_secret.clone())
                                    .or(this.context
                                        .payment_secret_confirm
                                        .is_not_empty()
                                        .then_some(this.context.wallet_secret_confirm.clone())
                                    );
                                this.context.payment_secret_score = payment_secret.map(secret_score);
                            }

                            if let Some(score) = this.context.payment_secret_score {
                                ui.label("");
                                ui.label(secret_score_to_text(score));
                                if score < 80.0 {
                                    ui.label("");
                                    ui.label(RichText::new(i18n("Passphrase is too weak")).color(egui::Color32::from_rgb(255, 120, 120)));
                                    if !core.settings.developer.disable_password_restrictions() {
                                        submit = false;
                                        ui.label(RichText::new(i18n("Please create a stronger passphrase")).color(egui::Color32::from_rgb(255, 120, 120)));
                                    }
                                }
                            }
                            ui.label("");

                            if this.context.payment_secret_confirm.is_not_empty() && this.context.payment_secret != this.context.payment_secret_confirm {
                                ui.label(" ");
                                ui.label(RichText::new(i18n("Passphrases do not match")).color(egui::Color32::from_rgb(255, 120, 120)));
                                ui.label(" ");
                                submit = false;
                            } else {
                                ui.label(" ");
                            }

                            if submit {
                                this.state = State::CreateWalletConfirm;
                                this.focus.clear();
                            }
                        }
                    })
                    .with_footer(|this,ui| {
                        if this.context.enable_payment_secret {
                            let is_weak = !core.settings.developer.disable_password_restrictions() && this.context.payment_secret_score.unwrap_or_default() < 80.0;
                            let enabled = this.context.wallet_secret == this.context.wallet_secret_confirm && this.context.wallet_secret.is_not_empty();
                            if ui.large_button_enabled(enabled && !is_weak, "Continue").clicked() {
                                this.state = State::CreateWalletConfirm;
                                this.focus.clear();
                            }
                        } else if ui.large_button_enabled(true, "Skip").clicked() {
                            this.state = State::CreateWalletConfirm;
                            this.focus.clear();
                        }

                    })
                    .render(ui);
            }
            State::CreateWalletConfirm => {
                // - TODO: present summary information
                self.state = State::CreateWallet;
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

                let args = self.context.clone();
                let wallet_create_result = Payload::<Result<(Arc<String>,AccountDescriptor)>>::new("wallet_create_result");
                if !wallet_create_result.is_pending() {

                    // TODO CREATE WALLET !
                    let wallet = self.runtime.wallet().clone();
                    spawn_with_result(&wallet_create_result, async move {

                        if args.enable_payment_secret && args.payment_secret.is_empty() {
                            return Err(Error::custom("Payment secret is empty"));
                        }

                        if args.enable_phishing_hint && args.phishing_hint.is_empty() {
                            return Err(Error::custom("Phishing hint is empty"));
                        }

                        let wallet_secret = Secret::from(args.wallet_secret);
                        let payment_secret = args.enable_payment_secret.then_some(Secret::from(args.payment_secret));

                        let wallet_args = WalletCreateArgs::new(
                            args.wallet_name.is_not_empty().then_some(args.wallet_name),
                            args.wallet_filename.is_not_empty().then_some(args.wallet_filename),
                            args.enable_phishing_hint.then_some(args.phishing_hint.into()),
                            // wallet_secret.clone(),
                            false
                        );
                        
                        wallet.clone().wallet_create(wallet_secret.clone(), wallet_args).await?;

                        let mnemonic = Mnemonic::random(args.word_count, Language::default())?;
                        let mnemonic_phrase_string = mnemonic.phrase_string();
                        // let account_kind = AccountKind::Bip32;

                        let prv_key_data_args = PrvKeyDataCreateArgs::new(
                            None,
                            payment_secret.clone(),
                            mnemonic_phrase_string.clone(),
                            // mnemonic.phrase_string(),
                            //payment_secret.clone(),
                            // - TODO
                            // WordCount::Words12.into(),
                        );

                        let prv_key_data_id = wallet.clone().prv_key_data_create(wallet_secret.clone(), prv_key_data_args).await?;

                        let account_create_args = AccountCreateArgs::new_bip32(
                            prv_key_data_id,
                            payment_secret.clone(),
                            args.account_name.is_not_empty().then_some(args.account_name),
                            None,
                            // account_kind,
                            // wallet_secret.clone(),
                            // payment_secret.clone(),
                        );

                        let account_descriptor = wallet.clone().accounts_create(wallet_secret, account_create_args).await?;

                        // let WalletCreateResponse { mnemonic, wallet_descriptor: _, account_descriptor, storage_descriptor: _ } = 
                        // wallet.wallet_create(wallet_secret, wallet_args, prv_key_data_args, account_args).await?;
                        Ok((Arc::new(mnemonic_phrase_string), account_descriptor))
                    });
                }

                if let Some(result) = wallet_create_result.take() {
                    match result {
                        Ok((mnemonic,account)) => {
                            self.context.zeroize();

                            println!("Wallet created successfully");
                            self.state = State::PresentMnemonic(mnemonic);
                            let device = core.device().clone();
                            core.get_mut::<modules::AccountManager>().select(Some(account.into()), device);
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
                    ui.label(RichText::new("Error creating a wallet").color(egui::Color32::from_rgb(255, 120, 120)));
                    ui.label(RichText::new(err.to_string()).color(egui::Color32::from_rgb(255, 120, 120)));
                    ui.label(" ");
                    ui.label(" ");

                    if ui.add_sized(editor_size, egui::Button::new("Restart")).clicked() {
                        this.state = State::Start;
                    }
                })
                .render(ui);
            }

            State::PresentMnemonic(mnemonic) => {
                let phrase = (*mnemonic).clone();

                Panel::new(self)
                    .with_caption("Private Key Mnemonic")
                    .with_body(|this,ui| {

                        let mut mnemonic_presenter = MnemonicPresenter::new(phrase.as_str(), &mut this.context.mnemonic_presenter_context);

                        ui.label(RichText::new(i18n(mnemonic_presenter.notice())).size(14.));
                        ui.label(" ");
                        ui.label(RichText::new("Never share your mnemonic with anyone!").color(Color32::LIGHT_RED));
                        ui.label(" ");
                        ui.label("Your default wallet private key mnemonic is:");
                        ui.label(" ");

                        mnemonic_presenter.render(ui);

                })
                .with_footer(|this,ui| {
                    if ui.add_sized(editor_size, egui::Button::new("Continue")).clicked() {
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
                        if ui.add_sized(editor_size, egui::Button::new("Continue")).clicked() {
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
                        if ui.add_sized(editor_size, egui::Button::new("Continue")).clicked() {
                            this.state = State::Start;
                            core.select::<modules::AccountManager>();
                            core.wallet_update_list();
                        }
                    })
                    .render(ui);
            }

        }

    }
}
