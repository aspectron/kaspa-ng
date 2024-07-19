use crate::imports::*;
use kaspa_wallet_core::{wallet::{AccountCreateArgs, PrvKeyDataCreateArgs, WalletCreateArgs}, encryption::EncryptionKind, api::{AccountsDiscoveryRequest, AccountsDiscoveryKind}};
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
    WalletMnemonic,
}

#[derive(Clone)]
pub enum State {
    Start,
    KeySelection,
    ImportSelection,
    ImportMnemonic,
    ImportMnemonicWithEditor,
    ImportMnemonicInteractive,
    ImportWallet,
    WalletName,
    AccountName,
    PhishingHint,
    WalletSecret,
    PaymentSecret,
    CreateWalletConfirm,
    CreateWallet,
    WalletError(Arc<Error>),
    PresentMnemonic(String),
    ConfirmMnemonic(String),
    Finish,
}

#[derive(Clone, Default)]
struct Context {
    word_count : WordCount,
    wallet_name: String,
    wallet_filename: String,
    account_name: String,
    enable_phishing_hint: bool,
    phishing_hint: String,
    wallet_secret: String,
    wallet_secret_confirm: String,
    wallet_secret_show: bool,
    wallet_secret_score: Option<f64>,
    enable_payment_secret: bool,
    payment_secret: String,
    payment_secret_confirm: String,
    payment_secret_show : bool,
    payment_secret_score: Option<f64>,
    mnemonic_presenter_context : MnemonicPresenterContext,
    import_private_key : bool,
    import_private_key_mnemonic : String,
    import_private_key_mnemonic_error : Option<String>,
    import_with_bip39_passphrase : bool,
    import_legacy : bool,
    import_advanced : bool,
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
        self.mnemonic_presenter_context.zeroize();

        self.import_private_key.zeroize();
        self.import_private_key_mnemonic.zeroize();
        self.import_private_key_mnemonic_error.zeroize();
        self.import_with_bip39_passphrase.zeroize();
        self.import_legacy.zeroize();
        self.import_advanced.zeroize();
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

                let mut back = false;

                Panel::new(self)
                    .with_caption(i18n("Create Wallet"))
                    .with_back_enabled(core.has_stack(), |_|{
                        back = true;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.add_space(64.);
                        ui.label(i18n("The following will guide you through the process of creating or importing a wallet."));
                        ui.label(" ");
                        ui.label(i18n("A wallet is stored in a file on your computer."));
                        ui.label(" ");
                        ui.label(i18n("You can create multiple wallets, but only one wallet can be open at a time."));
                    })
                    .with_handler(|this| {
                        this.state = State::KeySelection;
                    })
                    .render(ui);

                if back {
                    self.context.zeroize();
                    core.back();
                }
            }
            State::KeySelection => {

                let mut submit = false;
                let mut import = false;

                Panel::new(self)
                    .with_caption(i18n("Select Private Key Type"))
                    .with_back(|this| {
                        this.state = State::Start;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.add_space(64.);
                        ui.label(i18n("Please specify the private key type for the new wallet"));
                    })
                    .with_body(|this,ui| {

                        if ui.large_button(i18n("12 word mnemonic")).clicked() {
                            this.context.word_count = WordCount::Words12;
                            submit = true;
                        }
                        ui.label("");
                        if ui.large_button(i18n("24 word mnemonic")).clicked() {
                            this.context.word_count = WordCount::Words24;
                            submit = true;
                        }
                        // ui.label("");
                        // if ui.large_button_enabled(false,"MultiSig account").clicked() {
                        // }
                        ui.label("");
                        ui.separator();
                        ui.label("");
                        ui.label(i18n("Other operations"));
                        ui.label("");

                        if ui.large_button(i18n("Import existing")).clicked() {
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
                    .with_caption(i18n("Import Existing Private Key"))
                    .with_back(|this| {
                        this.state = State::KeySelection;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.add_space(64.);
                        ui.label(i18n("Please select the private key type you would like to import in the new wallet"));
                    })
                    .with_body(|this,ui| {
                        // ui.label("(You can import additional private keys later, once the wallet has been created)");

                        if ui.large_button(i18n("12 word mnemonic")).clicked() {
                            this.context.word_count = WordCount::Words12;
                            submit = true;
                        }

                        ui.label("");
                        ui.checkbox(&mut this.context.import_legacy, i18n("I have a 12 word mnemonic legacy account"));
                        ui.label(i18n("Select this option if your wallet was created"));
                        ui.label(i18n("using KDX or kaspanet.io web wallet"));
                        ui.label(RichText::new("NOT SUPPORTED IN THIS BETA RELEASE").size(10.).color(warning_color()));

                        if !this.context.import_legacy {
                            ui.label("");
                            if ui.large_button(i18n("24 word mnemonic")).clicked() {
                                this.context.word_count = WordCount::Words24;
                                submit = true;
                            }
                            // ui.label("");
                            // if ui.large_button("MultiSig account").clicked() {
                            // }

                            ui.label("");
                            ui.checkbox(&mut this.context.import_with_bip39_passphrase, i18n("Your mnemonic is protected with a bip39 passphrase"));

                            ui.label("");
                            ui.checkbox(&mut this.context.import_advanced, i18n("Advanced Options"));
                            if this.context.import_advanced {
                                ui.label("");
                                if ui.large_button_enabled(false, i18n("secp256k1 keypair")).clicked() {
                                    this.context.word_count = WordCount::Words12;
                                    submit = true;
                                }
                                ui.label(RichText::new("NOT SUPPORTED IN THIS BETA RELEASE").size(10.).color(warning_color()));
                            }
                        }



                    })
                    .with_footer(|_this,_ui| {
                    })
                    .render(ui);

                    if submit {
                        self.context.import_private_key = true;
                        self.state = State::WalletName;
                        self.focus.next(Focus::WalletName);
                    }
            }


            State::WalletName => {

                let wallet_exists = Rc::new(RefCell::new(false));

                Panel::new(self)
                    .with_caption(i18n("Wallet Name"))
                    .with_back(|this| {
                        this.state = State::KeySelection;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.add_space(64.);
                        ui.label(i18n("Please specify the name of the new wallet"));
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
                                ui.label(RichText::new(i18n("Enter wallet name")).size(12.).raised());
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
                            ui.label(format!("{} {}",i18n("Filename:"), this.context.wallet_filename));
                            ui.label(" ");

                            core.wallet_list().iter().for_each(|wallet| {
                                if wallet.filename == this.context.wallet_filename {
                                    *wallet_exists.borrow_mut() = true;
                                }
                            });

                            if *wallet_exists.borrow() {
                                ui.label(RichText::new(i18n("Wallet with this name already exists")).color(error_color()));
                                ui.label(" ");
                            }
                        }

                    })
                    .with_footer(|this,ui| {
                        if ui.large_button_enabled(this.context.wallet_name.is_not_empty() && !*wallet_exists.borrow(), i18n("Continue")).clicked() {
                            this.state = State::AccountName;
                            this.focus.next(Focus::AccountName);
                        }

                    })
                    .render(ui);
            }
            State::AccountName => {
                Panel::new(self)
                    .with_caption(i18n("Default Account Name"))
                    .with_back(|this| {
                        this.state = State::WalletName;
                        this.focus.next(Focus::WalletName);
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.heading(i18n("Optional"));
                        ui.label(" ");
                        ui.label(i18n("Please specify the name of the default account. The wallet will be created with a default account. Once created, you will be able to create additional accounts as you need."));
                        ui.label(" ");
                        ui.label(i18n("If not specified, the account will be represented by the numeric id."));
                    })
                    .with_body(|this,ui| {

                        TextEditor::new(
                            &mut this.context.account_name,
                            &mut this.focus,
                            Focus::AccountName,
                            |ui, text| {
                                // ui.add_space(8.);
                                ui.label(RichText::new(i18n("Enter first account name")).size(12.).raised());
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
                        let text = if this.context.account_name.is_not_empty() { i18n("Continue") } else { "Skip" };
                        if ui.large_button(i18n(text)).clicked() {
                            this.state = State::PhishingHint;
                            this.focus.next(Focus::PhishingHint);
                        }
                    })
                    .render(ui);
            }
            State::PhishingHint => {

                Panel::new(self)
                    .with_caption(i18n("Phishing Hint"))
                    .with_back(|this| {
                        this.state = State::AccountName;
                        this.focus.next(Focus::AccountName);
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.heading(i18n("Optional"));
                        ui.label(" ");
                        ui.label(i18n("\
                            'Phishing hint' is a secret word or a phrase that is displayed \
                            when you open your wallet. If you do not see the hint when opening \
                            your wallet, you may be accessing a fake wallet designed to steal \
                            your funds. If this occurs, stop using the wallet immediately, \
                            check the browser URL domain name and seek help on social networks \
                            (Kaspa Discord or Telegram)."));
                        ui.label(" ");
                    })
                    .with_body(|this,ui| {
                        TextEditor::new(
                            &mut this.context.phishing_hint,
                            &mut this.focus,
                            Focus::PhishingHint,
                            |ui, text| {
                                ui.label(RichText::new(i18n("Enter phishing hint")).size(12.).raised());
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
                        if ui.large_button(i18n("Continue")).clicked() {
                            this.state = State::WalletSecret;
                            this.focus.next(Focus::WalletSecret);
                        }
                    })
                    .render(ui);
            }
            State::WalletSecret => {

                Panel::new(self)
                    .with_caption(i18n("Wallet Encryption Password"))
                    .with_back(|this| {
                        this.state = State::PhishingHint;
                        this.focus.next(Focus::PhishingHint);
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.label(" ");
                        ui.label(i18n("Wallet password is used to encrypt your wallet data."));
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
                                ui.label(RichText::new(i18n("Enter wallet password")).size(12.).raised());
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
                                ui.label(RichText::new(i18n("Confirm wallet password")).size(12.).raised());
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
                        ui.checkbox(&mut this.context.wallet_secret_show, i18n("Show password"));
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
                            this.context.wallet_secret_score = wallet_secret.map(secret_score);
                        }

                        if let Some(score) = this.context.wallet_secret_score {
                            ui.label("");
                            ui.label(format!("{} {}",i18n("Secret score:"),secret_score_to_text(score)));
                            if score < 80.0 {
                                ui.label("");
                                ui.label(RichText::new(i18n("Secret is too weak")).color(error_color()));
                                if !core.settings.developer.password_restrictions_disabled() {
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
                        let is_weak = !core.settings.developer.password_restrictions_disabled() && this.context.wallet_secret_score.unwrap_or_default() < 80.0;
                        let enabled = this.context.wallet_secret == this.context.wallet_secret_confirm && this.context.wallet_secret.is_not_empty();
                        if ui.large_button_enabled(enabled && !is_weak, i18n("Continue")).clicked() {
                            this.state = State::PaymentSecret;
                            this.focus.next(Focus::PaymentSecret);
                        }
                    })
                    .render(ui);
            }
            State::PaymentSecret => {

                let mut proceed = self.context.import_private_key && !self.context.import_with_bip39_passphrase;

                Panel::new(self)
                    .with_caption(i18n("Payment & Recovery Password"))
                    .with_back(|this| {
                        this.state = State::WalletSecret;
                        this.focus.next(Focus::WalletSecret);
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|this,ui| {
                        if this.context.import_with_bip39_passphrase {

                        } else {
                            ui.heading(i18n("Optional"));
                            ui.label(" ");
                            ui.label(i18n("The optional payment & mnemonic recovery passphrase, known as BIP39 passphrase, if provided, will be required to \
                            send payments. This passphrase will also be required when recovering your wallet in addition to your mnemonic.\
                            If you loose or forget this passphrase, you will not \
                            be able to use mnemonic to recover your wallet!"));
                        }
                    })
                    .with_body(|this,ui| {
                        let mut submit = false;
                        let mut change = false;
    
    
                        if !this.context.import_with_bip39_passphrase {
                            ui.checkbox(&mut this.context.enable_payment_secret, i18n("Enable optional BIP39 passphrase"));
                        }

                        if this.context.enable_payment_secret || this.context.import_with_bip39_passphrase {
                            
                            ui.label("");

                            TextEditor::new(
                                &mut this.context.payment_secret,
                                &mut this.focus,
                                Focus::PaymentSecret,
                                |ui, text| {
                                    // ui.add_space(8.);
                                    ui.label(RichText::new(i18n("Enter BIP39 passphrase")).size(12.).raised());
                                    ui.add_sized(editor_size, TextEdit::singleline(text)
                                        .password(!this.context.payment_secret_show)
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
                                    ui.label(RichText::new(i18n("Confirm BIP39 passphrase")).size(12.).raised());
                                    ui.add_sized(editor_size, TextEdit::singleline(text)
                                        .password(!this.context.payment_secret_show)
                                        .vertical_align(Align::Center))
                                },
                            )
                            .submit(|_text,_focus| {
                                submit = true;
                            })
                            .build(ui);

                            ui.label("");
                            ui.checkbox(&mut this.context.payment_secret_show, i18n("Show passphrase"));
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
                                    ui.label(RichText::new(i18n("Passphrase is too weak")).color(warning_color()));
                                    // if !core.settings.developer.password_restrictions_disabled() {
                                    //     // submit = false;
                                    //     ui.label(RichText::new(i18n("Please create a stronger passphrase")).color(egui::Color32::from_rgb(255, 120, 120)));
                                    // }
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
                            let is_weak = !core.settings.developer.password_restrictions_disabled() && this.context.payment_secret_score.unwrap_or_default() < 80.0;
                            let enabled = this.context.wallet_secret == this.context.wallet_secret_confirm && this.context.wallet_secret.is_not_empty();
                            if ui.large_button_enabled(enabled && !is_weak, i18n("Continue")).clicked() {
                                proceed = true;
                            }
                        } else if ui.large_button_enabled(true, i18n("Skip")).clicked() {
                            proceed = true;
                        }

                    })
                    .render(ui);

                    if proceed {
                        if self.context.import_private_key {
                            self.state = State::ImportMnemonicWithEditor;
                        } else {
                            self.state = State::CreateWalletConfirm;
                        }
                        self.focus.clear();
                        
                    }

            }

            State::ImportMnemonic => {
                self.state = State::ImportMnemonicWithEditor;
                self.focus.next(Focus::WalletMnemonic);
            }

            State::ImportMnemonicWithEditor => {

                let mnemonic_is_ok = Rc::new(RefCell::new(false));
                let proceed = Rc::new(RefCell::new(false));

                Panel::new(self)
                    .with_caption(i18n("Mnemonic Import"))
                    .with_back(|this| {
                        this.state = State::KeySelection;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|this,ui| {
                        ui.add_space(64.);
                        match this.context.word_count {
                            WordCount::Words12 => {
                                ui.label(i18n("Please enter mnemonic comprised of 12 words"));
                            }
                            WordCount::Words24 => {
                                ui.label(i18n("Please enter mnemonic comprised of 24 words"));
                            }
                        }
                    })
                    .with_body(|this,ui| {
                        let mut submit = false;
                        TextEditor::new(
                            &mut this.context.import_private_key_mnemonic,
                            &mut this.focus,
                            Focus::WalletMnemonic,
                            |ui, text| {
                                // ui.add_space(8.);
                                ui.label(RichText::new(i18n("Enter mnemonic")).size(12.).raised());
                                let mut available_width = ui.available_width();
                                if available_width > 1024. {
                                    available_width *= 0.5;
                                } else if available_width > 512. {
                                    available_width *= 0.7;
                                }
                                ui.add_sized(vec2(available_width, 64.), TextEdit::multiline(text))
                            },
                        )
                        .submit(|_text,_focus| {
                            submit = true;
                        })
                        .build(ui);

                        let phrase = this.context.import_private_key_mnemonic.as_str().split_ascii_whitespace().filter(|s| s.is_not_empty()).collect::<Vec<&str>>();
                        let needed = match this.context.word_count {
                            WordCount::Words12 => 12,
                            WordCount::Words24 => 24,
                        } as usize;
                        // TODO - use comparison chain
                        #[allow(clippy::comparison_chain)]
                        if phrase.len() < needed {
                            ui.label("");
                            ui.label(format!("{} {} {}", i18n("Please enter additional"), needed - phrase.len(), i18n("words")));
                            ui.label("");
                        } else if phrase.len() > needed {
                            ui.label("");
                            ui.colored_label(error_color(), format!("{} '{}' {}", i18n("Too many words in the"), phrase.len() - needed, i18n("word mnemonic")));
                            ui.label("");
                        } else {
                            let mut phrase = phrase.join(" ");
                            match Mnemonic::new(phrase.as_str(), Language::default()) {
                                Ok(_) => {
                                    *mnemonic_is_ok.borrow_mut() = true;
                                    if submit {
                                        *proceed.borrow_mut() = true;
                                    }
                                },
                                Err(err) => {
                                    phrase.zeroize();
                                    ui.label("");
                                    ui.label(RichText::new(format!("Error processing mnemonic; {err}")).color(error_color()));
                                    ui.label("");
                                }
                            }
                        }
                    })
                    .with_footer(|_this,ui| {
                        if ui.large_button_enabled(*mnemonic_is_ok.borrow(), i18n("Continue")).clicked() {
                            *proceed.borrow_mut() = true;
                        }
                    })
                    .render(ui);

                if *proceed.borrow() {
                    self.state = State::ImportWallet;
                    self.focus.clear();
                }

            }

            State::ImportMnemonicInteractive => {
                // TODO
            }

            State::ImportWallet => {

                Panel::new(self)
                    .with_caption(i18n("Importing Wallet"))
                    .with_header(|_, ui|{
                        ui.label(" ");
                        ui.label(i18n("Please wait..."));
                        ui.label(" ");
                        ui.label(" ");
                        ui.add_space(64.);
                        ui.add(egui::Spinner::new().size(92.));
                    })
                    .render(ui);

                let mut args = self.context.clone();
                let wallet_import_result = Payload::<Result<Vec<AccountDescriptor>>>::new("wallet_import_result");
                if !wallet_import_result.is_pending() {

                    let wallet = self.runtime.wallet().clone();
                    let import_legacy = self.context.import_legacy;
                    spawn_with_result(&wallet_import_result, async move {

                        if args.import_with_bip39_passphrase && args.payment_secret.is_empty() {
                            return Err(Error::custom(i18n("Payment secret is empty")));
                        }

                        if args.enable_phishing_hint && args.phishing_hint.is_empty() {
                            return Err(Error::custom(i18n("Phishing hint is empty")));
                        }

                        let wallet_secret = Secret::from(args.wallet_secret.as_str());
                        let payment_secret = args.import_with_bip39_passphrase.then_some(Secret::from(args.payment_secret.as_str()));
                        let mnemonic = Secret::from(sanitize_mnemonic(args.import_private_key_mnemonic.as_str()));


                        let request = AccountsDiscoveryRequest {
                            discovery_kind: AccountsDiscoveryKind::Bip44,
                            address_scan_extent: 32,
                            account_scan_extent: 16,
                            bip39_passphrase: payment_secret.clone(),
                            bip39_mnemonic: mnemonic.clone(),
                        };

                        let response = wallet.clone().accounts_discovery_call(request).await?;
                        let number_of_accounts = (response.last_account_index_found + 1) as usize;

                        wallet.clone().batch().await?;

                        let wallet_args = WalletCreateArgs::new(
                            args.wallet_name.is_not_empty().then_some(args.wallet_name.clone()),
                            args.wallet_filename.is_not_empty().then_some(args.wallet_filename.clone()),
                            EncryptionKind::XChaCha20Poly1305,
                            args.enable_phishing_hint.then_some(args.phishing_hint.as_str().into()),
                            false
                        );
                        
                        wallet.clone().wallet_create(wallet_secret.clone(), wallet_args).await?;

                        let prv_key_data_args = PrvKeyDataCreateArgs::new(
                            None,
                            payment_secret.clone(),
                            mnemonic,
                        );

                        let prv_key_data_id = wallet.clone().prv_key_data_create(wallet_secret.clone(), prv_key_data_args).await?;

                        let mut account_descriptors = Vec::with_capacity(number_of_accounts);
                        if import_legacy{
                            for _account_index in 0..number_of_accounts {
                                let account_create_args = AccountCreateArgs::new_legacy(
                                    prv_key_data_id,
                                    args.account_name.is_not_empty().then_some(args.account_name.clone()),
                                );
                                account_descriptors.push(wallet.clone().accounts_create(wallet_secret.clone(), account_create_args).await?);
                            }
                        }else{
                            for account_index in 0..number_of_accounts {
                                let account_create_args = AccountCreateArgs::new_bip32(
                                    prv_key_data_id,
                                    payment_secret.clone(),
                                    args.account_name.is_not_empty().then_some(args.account_name.clone()),
                                    Some(account_index as u64),
                                );
                                account_descriptors.push(wallet.clone().accounts_create(wallet_secret.clone(), account_create_args).await?);
                            }
                        }

                        wallet.clone().flush(wallet_secret).await?;

                        args.zeroize();

                        Ok(account_descriptors)
                    });
                }

                if let Some(result) = wallet_import_result.take() {
                    match result {
                        Ok(account_descriptors) => {
                            self.context.zeroize();
                            core.handle_account_creation(account_descriptors);
                            self.state = State::Finish;
                        }
                        Err(err) => {
                            log_error!("{} {}",i18n("Wallet creation error:"), err);
                            self.state = State::WalletError(Arc::new(err));
                        }
                    }
                }
            }

            State::CreateWalletConfirm => {
                // - TODO: present summary information
                self.state = State::CreateWallet;
            }
            State::CreateWallet => {

                Panel::new(self)
                    .with_caption(i18n("Creating Wallet"))
                    .with_header(|_, ui|{
                        ui.label(" ");
                        ui.label(i18n("Please wait..."));
                        ui.label(" ");
                        ui.label(" ");
                        ui.add_space(64.);
                        ui.add(egui::Spinner::new().size(92.));
                    })
                    .render(ui);

                let args = self.context.clone();
                let wallet_create_result = Payload::<Result<(String,AccountDescriptor)>>::new("wallet_create_result");
                if !wallet_create_result.is_pending() {

                    let wallet = self.runtime.wallet().clone();
                    spawn_with_result(&wallet_create_result, async move {

                        if args.enable_payment_secret && args.payment_secret.is_empty() {
                            return Err(Error::custom(i18n("Payment secret is empty")));
                        }

                        if args.enable_phishing_hint && args.phishing_hint.is_empty() {
                            return Err(Error::custom(i18n("Phishing hint is empty")));
                        }

                        let wallet_secret = Secret::from(args.wallet_secret);
                        let payment_secret = args.enable_payment_secret.then_some(Secret::from(args.payment_secret));

                        wallet.clone().batch().await?;

                        let wallet_args = WalletCreateArgs::new(
                            args.wallet_name.is_not_empty().then_some(args.wallet_name),
                            args.wallet_filename.is_not_empty().then_some(args.wallet_filename),
                            EncryptionKind::XChaCha20Poly1305,
                            args.enable_phishing_hint.then_some(args.phishing_hint.into()),
                            false
                        );
                        
                        wallet.clone().wallet_create(wallet_secret.clone(), wallet_args).await?;

                        let mnemonic = Mnemonic::random(args.word_count, Language::default())?;
                        let mnemonic_phrase_string = mnemonic.phrase_string();
                        let prv_key_data_args = PrvKeyDataCreateArgs::new(
                            None,
                            payment_secret.clone(),
                            Secret::from(mnemonic_phrase_string.clone()),
                        );

                        let prv_key_data_id = wallet.clone().prv_key_data_create(wallet_secret.clone(), prv_key_data_args).await?;

                        let account_create_args = AccountCreateArgs::new_bip32(
                            prv_key_data_id,
                            payment_secret.clone(),
                            args.account_name.is_not_empty().then_some(args.account_name),
                            None,
                        );

                        let account_descriptor = wallet.clone().accounts_create(wallet_secret.clone(), account_create_args).await?;

                        wallet.clone().flush(wallet_secret).await?;

                        Ok((mnemonic_phrase_string, account_descriptor))
                    });
                }

                if let Some(result) = wallet_create_result.take() {
                    match result {
                        Ok((mnemonic,account_descriptor)) => {
                            self.context.zeroize();
                            core.handle_account_creation(vec![account_descriptor]);
                            self.state = State::PresentMnemonic(mnemonic);
                        }
                        Err(err) => {
                            log_error!("{} {}",i18n("Wallet creation error:"), err);
                            self.state = State::WalletError(Arc::new(err));
                        }
                    }
                }

            }

            State::WalletError(err) => {

                Panel::new(self)
                .with_caption(i18n("Error"))
                .with_header(move |this,ui| {
                    ui.label(" ");
                    ui.label(" ");
                    ui.label(RichText::new(i18n("Error creating a wallet")).color(egui::Color32::from_rgb(255, 120, 120)));
                    ui.label(RichText::new(err.to_string()).color(egui::Color32::from_rgb(255, 120, 120)));
                    ui.label(" ");
                    ui.label(" ");

                    if ui.large_button(i18n("Restart")).clicked() {
                        this.state = State::Start;
                    }
                })
                .render(ui);
            }

            State::PresentMnemonic(mut mnemonic) => {

                let mut finish = false;

                Panel::new(self)
                    .with_caption(i18n("Private Key Mnemonic"))
                    .with_body(|this,ui| {

                        let mut mnemonic_presenter = MnemonicPresenter::new(mnemonic.as_str(), &mut this.context.mnemonic_presenter_context);

                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(i18n(mnemonic_presenter.notice())).size(14.));
                            ui.label(RichText::new(i18n(mnemonic_presenter.warning())).size(14.).color(theme_color().warning_color));
                        });

                        ui.label("");
                        mnemonic_presenter.render(ui, Some(i18n("Your default wallet private key mnemonic is:")));
                        ui.label("");
                })
                .with_footer(|_this,ui| {
                    if ui.large_button_enabled(true, i18n("Continue")).clicked() {
                        finish = true;
                    }
                })
                .render(ui);
            
            if finish {
                    // this.state = State::ConfirmMnemonic(mnemonic);
                    mnemonic.zeroize();
                    self.state = State::Finish;
                }

            }

            State::ConfirmMnemonic(mnemonic) => {
                Panel::new(self)
                    .with_caption(i18n("Confirm Mnemonic"))
                    .with_back(|this|{
                        this.state = State::PresentMnemonic(mnemonic);
                    })
                    .with_header(|_this,ui| {
                        ui.label(i18n("Please validate your mnemonic"));
                    })
                    .with_body(|_this,_ui| {
                        // TODO
                    })
                    .with_footer(move |this,ui| {
                        if ui.add_sized(editor_size, egui::Button::new(i18n("Continue"))).clicked() {
                            this.state = State::Finish;
                        }
                    })
                    .render(ui);
            }

            State::Finish => {

                Panel::new(self)
                    .with_caption(i18n("Wallet Created"))
                    .with_body(|_this,ui| {
                        ui.label(" ");
                        ui.label(i18n("Your wallet has been created and is ready to use."));
                        ui.label(" ");
                    })
                    .with_footer(move |this,ui| {
                        if ui.large_button(i18n("Continue")).clicked() {
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
