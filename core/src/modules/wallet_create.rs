use crate::imports::*;
use kaspa_wallet_core::{api::{AccountsDiscoveryKind, AccountsDiscoveryRequest}, encryption::EncryptionKind, storage::keydata::PrvKeyDataVariantKind, wallet::{AccountCreateArgs, PrvKeyDataCreateArgs, WalletCreateArgs}};
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
    DecryptWalletSecret,
}

#[derive(Clone)]
pub enum KeyOperationKind {
    Create,
    ImportKey,
    ImportFile,
}

#[derive(Clone)]
pub enum State {
    Start,
    KeySelection,
    ImportSelection,
    ImportMnemonic,
    ImportMnemonicWithEditor,
    ImportMnemonicInteractive,
    WalletFileSecret,
    DecryptWalletFile,
    ImportWallet,
    // WalletName { kind: KeyOperationKind },
    WalletName,
    AccountName,
    PhishingHint,
    WalletSecret,
    PaymentSecret,
    CreateWalletConfirm,
    CreateWallet,
    WalletError(Arc<Error>, Arc<State>),
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
    decrypt_wallet_secret: String,
    wallet_secret_confirm: String,
    wallet_secret_show: bool,
    wallet_secret_score: Option<f64>,
    enable_payment_secret: bool,
    payment_secret: String,
    payment_secret_confirm: String,
    payment_secret_show : bool,
    payment_secret_score: Option<f64>,
    payment_secret_submitted: bool,
    mnemonic_presenter_context : MnemonicPresenterContext,
    import_private_key : bool,
    import_private_key_file: bool,
    import_private_key_mnemonic : String,
    import_private_key_mnemonic_error : Option<String>,
    import_with_bip39_passphrase : bool,
    import_legacy : bool,
    import_advanced : bool,
    wallet_file_data: Option<WalletFileData>
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
        self.decrypt_wallet_secret.zeroize();
        self.import_legacy.zeroize();
        self.import_advanced.zeroize();
        self.payment_secret_submitted = false;
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
    
    pub fn import_selection<M>(context:&mut M, word_count: &mut WordCount, import_legacy: &mut bool, bip39_passphrase: &mut bool, ui: &mut Ui, back_callback:Option<impl FnOnce(&mut M)>)->bool{
        let mut submit = false;
        if *import_legacy {
            *bip39_passphrase = false;
        }
        let mut panel = Panel::new(context)
            .with_caption(i18n("Import Existing Private Key"))
            .with_close_enabled(false, |_|{
            })
            .with_header(|_ctx,ui| {
                ui.add_space(64.);
                ui.label(i18n("Please select the private key type you would like to import in the new wallet"));
            })
            .with_body(|_this,ui| {
                // ui.label("(You can import additional private keys later, once the wallet has been created)");
                // let word_12_selected = !*import_legacy && *word_count == WordCount::Words12;
                // if ui.large_selected_button(word_12_selected, i18n("12 word mnemonic")).clicked() {
                if ui.large_button(i18n("12 word mnemonic")).clicked() {
                    *word_count = WordCount::Words12;
                    *import_legacy = false;
                    submit = true;
                }
                ui.label("");

                // if ui.large_selected_button(*word_count == WordCount::Words24, i18n("24 word mnemonic")).clicked() {
                if ui.large_button(i18n("24 word mnemonic")).clicked() {
                    *word_count = WordCount::Words24;
                    *import_legacy = false;
                    submit = true;
                }
                ui.label("");
                ui.add_enabled_ui(!*import_legacy, |ui|{
                    ui.checkbox(bip39_passphrase, i18n("Your mnemonic is protected with a bip39 passphrase"));
                });
                ui.label("");

                ui.medium_separator();
                ui.label("");

                ui.label(i18n("Select this option if your wallet was created"));
                ui.label(i18n("using KDX or kaspanet.io web wallet"));
                ui.label("");
                // if ui.large_selected_button(*import_legacy, format!("    {}    ", i18n("Legacy 12 word mnemonic"))).clicked() {
                if ui.large_button_enabled(!*bip39_passphrase,format!("    {}    ", i18n("Legacy 12 word mnemonic"))).clicked() {
                    *word_count = WordCount::Words12;
                    *import_legacy = true;
                    *bip39_passphrase = false;
                    submit = true;
                }
                // ui.label("");

                // ui.medium_separator();
                // ui.label("");
                
                // if ui.large_button(i18n("Continue")).clicked() {
                //     submit = true;
                // }
            })
            .with_footer(|_this,_ui| {
            });

            if let Some(back_callback) = back_callback{
                panel = panel.with_back(back_callback)
            }

            panel.render(ui);
        submit
    }

    pub fn import_mnemonic<F:core::marker::Copy + Eq + PartialEq + core::fmt::Debug, M>(context: &mut M, mnemonic_phrase: &mut String, word_count: &WordCount, focus_manager: &mut FocusManager<F>, focus_value: F, ui: &mut Ui, back_callback: impl FnOnce(&mut M))->bool{
        let mnemonic_is_ok = Rc::new(RefCell::new(false));
        let proceed = Rc::new(RefCell::new(false));
        let needed = match word_count {
            WordCount::Words12 => 12,
            WordCount::Words24 => 24,
        } as usize;
        Panel::new(context)
            .with_caption(i18n("Mnemonic Import"))
            .with_back(back_callback)
            .with_close_enabled(false, |_|{
            })
            .with_header(|_this,ui| {
                ui.add_space(64.);
                ui.label(i18n_args("Please enter mnemonic comprised of {number} words", &[("number", needed.to_string())]));
            })
            .with_body(|_this,ui| {
                let mut submit = false;
                TextEditor::new(
                    mnemonic_phrase,
                    focus_manager,
                    focus_value,
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

                let phrase = mnemonic_phrase.as_str().split_ascii_whitespace().filter(|s| s.is_not_empty()).collect::<Vec<&str>>();
                // TODO - use comparison chain
                #[allow(clippy::comparison_chain)]
                if phrase.len() < needed {
                    ui.label("");
                    ui.label(i18n_args("Please enter additional {amount} words", &[("amount", (needed - phrase.len()).to_string())]));
                    ui.label("");
                } else if phrase.len() > needed {
                    ui.label("");
                    ui.colored_label(error_color(), i18n_args("Too many words ({words}) in the {amount} word mnemonic", &[
                        ("words",phrase.len().to_string()),
                        ("amount",(phrase.len() - needed).to_string())
                    ]));
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
                            ui.label(RichText::new(i18n_args("Error processing mnemonic: {err}",&[("err",err.to_string())])).color(error_color()));
                            if matches!(err,kaspa_bip32::Error::Bip39) {
                                ui.label(RichText::new(i18n("Your mnemonic phrase is invalid")).color(error_color()));
                            }
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
        let proceed = *proceed.borrow();
        proceed
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

                self.context.import_with_bip39_passphrase = false;
                self.context.import_legacy = false;
                self.context.import_private_key = false;
                self.context.import_private_key_file = false;

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
                        let mut submit = false;
                        if ui.large_button(i18n("12 word mnemonic")).clicked() {
                            this.context.word_count = WordCount::Words12;
                            submit = true;
                        }
                        ui.label("");
                        if ui.large_button(i18n("24 word mnemonic")).clicked() {
                            this.context.word_count = WordCount::Words24;
                            submit = true;
                        }

                        if submit {
                            // this.state = State::WalletName { kind : KeyOperationKind::Create };
                            this.state = State::WalletName;
                            this.focus.next(Focus::WalletName);
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
                            this.state = State::ImportSelection;
                            this.focus.clear();
                        }
                        ui.label("");

                        if ui.large_button(i18n("Import existing file")).clicked() {
                            this.context.import_private_key_file = true;
                            this.state = State::ImportMnemonicInteractive;
                        }
                        ui.label("");

                    })
                    .with_footer(|_this,_ui| {
                    })
                    .render(ui);
            }
            State::ImportSelection => {
                let submit = Self::import_selection::<State>(&mut self.state,
                    &mut self.context.word_count,
                    &mut self.context.import_legacy,
                    &mut self.context.import_with_bip39_passphrase,
                    ui,
                    Some(|state : &mut State|{
                        *state = State::KeySelection
                    })
                );
                if submit {
                    self.context.import_private_key = true;
                    // self.state = State::WalletName { kind : KeyOperationKind::ImportKey };
                    self.state = State::WalletName;
                    self.focus.next(Focus::WalletName);
                }
            }
            // State::WalletName { kind } => {
            State::WalletName => {

                let wallet_exists = Rc::new(RefCell::new(false));

                Panel::new(self)
                    .with_caption(i18n("Wallet Name"))
                    .with_back(|this| {
                        // match kind {
                        //     KeyOperationKind::Create => {
                        //         this.state = State::KeySelection;
                        //     }
                        //     KeyOperationKind::Import => {
                        //         this.state = State::ImportSelection;
                        //     }
                        // }
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
                        let text = if this.context.account_name.is_not_empty() { i18n("Continue") } else { i18n("Skip") };
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

                let mut proceed = self.context.import_legacy || (self.context.import_private_key && !self.context.import_with_bip39_passphrase);
                let mut continue_or_skip = false;
                if !self.context.import_legacy {
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
        
        
                            if !this.context.import_with_bip39_passphrase && ui.checkbox(&mut this.context.enable_payment_secret, i18n("Enable optional BIP39 passphrase")).changed() {
                                 this.context.payment_secret_submitted = false;
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
                                    this.context.payment_secret_submitted = false;
                                    let payment_secret = this
                                        .context
                                        .payment_secret
                                        .is_not_empty()
                                        .then_some(this.context.payment_secret.clone())
                                        .or(this.context
                                            .payment_secret_confirm
                                            .is_not_empty()
                                            .then_some(this.context.payment_secret_confirm.clone())
                                        );
                                    this.context.payment_secret_score = payment_secret.map(secret_score);
                                }

                                if let Some(score) = this.context.payment_secret_score {
                                    ui.label("");
                                    ui.label(secret_score_to_text(score));
                                    if score < 80.0 {
                                        ui.label("");
                                        ui.label(RichText::new(i18n("Passphrase is too weak")).color(warning_color()));
                                        if !core.settings.developer.password_restrictions_disabled() {
                                            submit = false;
                                            //ui.label(RichText::new(i18n("Please create a stronger passphrase")).color(egui::Color32::from_rgb(255, 120, 120)));
                                        }
                                    }
                                }
                                ui.label("");

                                if this.context.payment_secret.is_not_empty() && this.context.payment_secret != this.context.payment_secret_confirm {
                                    ui.label(" ");
                                    ui.label(RichText::new(i18n("Passphrases do not match")).color(egui::Color32::from_rgb(255, 120, 120)));
                                    ui.label(" ");
                                    submit = false;
                                } else {
                                    ui.label(" ");
                                }

                                if submit {
                                    if this.context.payment_secret.is_empty() {
                                        this.context.payment_secret_submitted = true;
                                    } else if this.context.import_with_bip39_passphrase {
                                        proceed = true;
                                    } else {
                                        this.state = State::CreateWalletConfirm;
                                        this.focus.clear();
                                    }
                                }
                                if this.context.payment_secret_submitted {
                                    ui.label(RichText::new(i18n("Please provide BIP39 passphrase.")).color(egui::Color32::from_rgb(255, 120, 120)));
                                }
                            }
                        })
                        .with_footer(|this,ui| {

                            if this.context.enable_payment_secret || this.context.import_with_bip39_passphrase {
                                let is_weak = !core.settings.developer.password_restrictions_disabled() && this.context.payment_secret_score.unwrap_or_default() < 80.0;
                                let enabled = this.context.payment_secret == this.context.payment_secret_confirm && this.context.payment_secret.is_not_empty();
                                if ui.large_button_enabled(enabled && !is_weak, i18n("Continue")).clicked() {
                                    continue_or_skip = true;
                                }
                            } else if ui.large_button_enabled(true, i18n("Skip")).clicked() {
                                this.context.payment_secret.zeroize();
                                continue_or_skip = true;
                            }

                        })
                        .render(ui);
                }

                if proceed || continue_or_skip{
                    if self.context.import_private_key_file {
                        self.state = State::ImportWallet;
                        self.focus.clear();
                    } else if self.context.import_private_key {
                        self.state = State::ImportMnemonic;
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
                let proceed = Self::import_mnemonic::<Focus, State>(
                    &mut self.state,
                    &mut self.context.import_private_key_mnemonic,
                    &self.context.word_count,
                    &mut self.focus,
                    Focus::WalletMnemonic,
                    ui,
                    |m|{
                        *m = State::KeySelection;
                    }
                );

                if proceed {
                    self.state = State::ImportWallet;
                    self.focus.clear();
                }

            }

            State::ImportMnemonicInteractive => {
                Panel::new(self)
                    .with_caption(i18n("Importing Wallet File"))
                    .with_header(|_, ui|{
                        ui.label(" ");
                        ui.label(i18n("Select file..."));
                        ui.label(" ");
                        ui.label(" ");
                        ui.add_space(64.);
                        ui.add(egui::Spinner::new().size(92.));
                    })
                    .render(ui);
                
                let wallet_import_result = Payload::<Result<Option<String>>>::new("wallet_import_file_dialog");
                if !wallet_import_result.is_pending() {
                    wallet_import_result.mark_pending();
                    let import_result = wallet_import_result.clone();
                    let file_handle = rfd::AsyncFileDialog::new()
                        .add_filter("LegacyWallet", &["kpk"])
                        .add_filter("GolangWallet", &["json"])
                        .set_directory("/")
                        .pick_file();
                    #[cfg(target_arch="wasm32")]
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Some(file_handle) = file_handle.await{
                            let file_data = file_handle.read().await;
                            let json = String::from_utf8_lossy(&file_data.clone()).to_string();
                            log_trace!("json: {json}");
                            import_result.store(Ok(Some(json)));
                        }else{
                            import_result.store(Ok(None));
                        }
                    });
                    #[cfg(not(target_arch="wasm32"))]
                    spawn_with_result(&import_result, async move {
                        if let Some(file_handle) = file_handle.await{
                            let file_data = file_handle.read().await;
                            let json = String::from_utf8_lossy(&file_data.clone()).to_string();
                            log_trace!("json: {json}");
                            Ok(Some(json))
                        }else{
                            Ok(None)
                        }
                    });
                }

                if let Some(result) = wallet_import_result.take() {
                    match result {
                        Ok(json) => {
                            if let Some(json) = json{
                                match parse_wallet_file(&json){
                                    Ok(data)=>{
                                        self.context.wallet_file_data = Some(data);
                                        self.state = State::WalletFileSecret;
                                    }
                                    Err(err)=>{
                                        log_error!("{} {}",i18n("Wallet import error:"), err);
                                        self.state = State::WalletError(Arc::new(err), State::ImportMnemonicInteractive.into());
                                    }
                                }
                            }else{
                                self.state = State::KeySelection;
                            }
                        }
                        Err(err) => {
                            log_error!("{} {}",i18n("Wallet import error:"), err);
                            self.state = State::WalletError(Arc::new(err), State::ImportMnemonicInteractive.into());
                        }
                    }
                }
            }

            State::WalletFileSecret =>{
                let data = self.context.wallet_file_data.as_ref().unwrap().clone();
                Panel::new(self)
                .with_caption(i18n("Wallet File Secret"))
                .with_back(|this|{
                    this.state = State::ImportMnemonicInteractive;
                })
                .with_header(move |_this,ui| {
                    ui.label(" ");
                    ui.label(i18n("For decrypting uploaded file please enter wallet secret."));
                    ui.label(" ");
                })
                .with_body(|this, ui|{
                    if core.settings.developer.enable {
                        CollapsingHeader::new(i18n("File contents"))
                            .default_open(false)
                            .show_unindented(ui, |ui| {
                                ui.label(data.to_string());
                                ui.label(" ");
                            });
                    }
                    TextEditor::new(
                        &mut this.context.decrypt_wallet_secret,
                        &mut this.focus,
                        Focus::DecryptWalletSecret,
                        |ui, text| {
                            ui.label(RichText::new(i18n("Enter password to decrypt this wallet file")).size(12.).raised());
                            ui.add_sized(editor_size, TextEdit::singleline(text).password(true)
                                .vertical_align(Align::Center))
                        },
                    )
                    .submit(|text,focus| {
                        if text.is_not_empty(){
                            this.state = State::DecryptWalletFile;
                            focus.clear();
                        }
                    })
                    .build(ui);
                    ui.label(" ");
                    if ui.large_button_enabled(this.context.decrypt_wallet_secret.is_not_empty(), i18n("Decrypt")).clicked(){
                        this.state = State::DecryptWalletFile;
                    }
                    ui.label(" ");
                    if ui.large_button(i18n("Cancel")).clicked(){
                        this.state = State::Start;
                        this.context.decrypt_wallet_secret.zeroize();
                        this.context.wallet_file_data = None;
                        if core.has_stack() {
                            core.back();
                        }
                    }
                })
                .render(ui);
            }

            State::DecryptWalletFile=>{
                Panel::new(self)
                    .with_caption(i18n("Decrypting Wallet File"))
                    .with_header(|_, ui|{
                        ui.label(" ");
                        ui.label(i18n("Please wait..."));
                        ui.label(" ");
                        ui.label(" ");
                        ui.add_space(64.);
                        ui.add(egui::Spinner::new().size(92.));
                    })
                    .render(ui);
                let wallet_file_data = self.context.wallet_file_data.as_ref().unwrap().clone();
                let import_secret = Secret::from(self.context.decrypt_wallet_secret.as_str());
                let wallet_decrypt_result = Payload::<Result<WalletFileDecryptedData>>::new("wallet_file_decrypt_result");
                if !wallet_decrypt_result.is_pending() {
                    //let wallet = self.runtime.wallet().clone();
                    spawn_with_result(&wallet_decrypt_result, async move {
                        sleep(Duration::from_secs(2)).await;
    
                        match wallet_file_data{
                            WalletFileData::Legacy(data)=>{
                                let key_data = kaspa_wallet_core::compat::gen0::get_v0_keydata(&data, &import_secret)?;
                                Ok(WalletFileDecryptedData::Legacy(key_data.mnemonic.clone()))
                            }
                            WalletFileData::GoWallet(data)=>{
                                let mnemonic = match data {
                                    WalletType::SingleV0(data)=>{
                                        kaspa_wallet_core::compat::gen1::decrypt_mnemonic(
                                            data.num_threads,
                                            data.encrypted_mnemonic,
                                            import_secret.as_ref()
                                        )?
                                    }
                                };
                                Ok(WalletFileDecryptedData::Core(mnemonic))
                            }
                            WalletFileData::Core(_data)=>{
                                Err(Error::custom("Core wallet import not supported yet."))
                            }
                        }
                    })
                }

                if let Some(result) = wallet_decrypt_result.take() {
                    self.context.decrypt_wallet_secret.zeroize(); 
                    match result {
                        Ok(wallet_file_decrypted_data) => {
                            match wallet_file_decrypted_data{
                                WalletFileDecryptedData::Legacy(mnemonic)=>{
                                    self.context.word_count = WordCount::Words12;
                                    self.context.import_legacy = true;
                                    self.context.import_with_bip39_passphrase = false;
                                    self.context.import_private_key_mnemonic = mnemonic;
                                    self.state = State::WalletName;
                                }
                                WalletFileDecryptedData::Core(mnemonic)=>{
                                    let size = mnemonic.trim().split(' ').collect::<Vec<_>>().len();
                                    match size{
                                        24|12=>{
                                            if size == 12 {
                                                self.context.word_count = WordCount::Words12;
                                            }else{
                                                self.context.word_count = WordCount::Words24;
                                            }
                                            self.context.import_legacy = false;
                                            self.context.import_with_bip39_passphrase = false;
                                            self.context.import_private_key_mnemonic = mnemonic;
                                            self.state = State::WalletName;
                                        }
                                        _=>{
                                            self.state = State::WalletError(Arc::new(Error::Custom(format!("Invalid mnemonic count: {size}"))), State::ImportMnemonicInteractive.into());
                                        }
                                    }
                                    
                                }
                            }
                        }
                        Err(err) => {
                            log_error!("{} {}",i18n("Wallet decrypting error:"), err);
                            self.state = State::WalletError(Arc::new(err), State::WalletFileSecret.into());
                        }
                    }
                }
            }

            State::ImportWallet => {
                let import_legacy = self.context.import_legacy;
                let caption = if import_legacy {
                    i18n("Importing Legacy Wallet")
                }else{
                    i18n("Importing Wallet")
                };
                Panel::new(self)
                    .with_caption(caption)
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
                            PrvKeyDataVariantKind::Mnemonic,
                        );

                        let prv_key_data_id = wallet.clone().prv_key_data_create(wallet_secret.clone(), prv_key_data_args).await?;

                        let mut account_descriptors = Vec::with_capacity(number_of_accounts);
                        if import_legacy{
                            for _account_index in 0..number_of_accounts {
                                let account_create_args = AccountCreateArgs::new_legacy(
                                    prv_key_data_id,
                                    args.account_name.is_not_empty().then_some(args.account_name.clone()),
                                );
                                account_descriptors.push(wallet.clone().accounts_import(wallet_secret.clone(), account_create_args).await?);
                            }
                        }else{
                            for account_index in 0..number_of_accounts {
                                let account_create_args = AccountCreateArgs::new_bip32(
                                    prv_key_data_id,
                                    payment_secret.clone(),
                                    args.account_name.is_not_empty().then_some(args.account_name.clone()),
                                    Some(account_index as u64),
                                );
                                // log_info!("account_create_args: {:?}", account_create_args);
                                account_descriptors.push(wallet.clone().accounts_import(wallet_secret.clone(), account_create_args).await?);
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
                            self.state = State::WalletError(Arc::new(err), State::Start.into());
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
                            PrvKeyDataVariantKind::Mnemonic,
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
                            self.state = State::WalletError(Arc::new(err), State::Start.into());
                        }
                    }
                }

            }
            State::WalletError(err, back_state) => {
                let msg = if self.context.import_private_key || self.context.import_private_key_file{i18n("Error importing a wallet")}else{i18n("Error creating a wallet")};
                Panel::new(self)
                .with_caption(i18n("Error"))
                .with_header(move |this,ui| {
                    ui.label(" ");
                    ui.label(" ");
                    ui.label(RichText::new(msg).color(egui::Color32::from_rgb(255, 120, 120)));
                    ui.label(RichText::new(err.to_string()).color(egui::Color32::from_rgb(255, 120, 120)));
                    ui.label(" ");
                    ui.label(" ");

                    if ui.large_button(i18n("Restart")).clicked() {
                        this.state = back_state.as_ref().clone();
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
