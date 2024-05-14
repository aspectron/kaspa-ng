use crate::imports::*;

#[derive(Clone)]
pub enum CreateAccountKind {
    Bip44,
    Bip32,
    Legacy,
    MultiSig,
    Keypair,
    // Keypair,
    // MultiSig,
}


#[derive(Clone)]
pub enum State {
    Start,
    PrivateKeyCreate,
    PrivateKeyConfirm,
    AccountName,
    WalletSecret,
    PaymentSecret,
    CreateAccount,
    AccountError(Arc<Error>),
    PresentMnemonic(Arc<CreationData>),
    ConfirmMnemonic(Arc<CreationData>),
    Finish(Arc<dyn CoreAccount>),
}

#[derive(Clone)]
pub enum CreationData {
    Bip44 {
        name : Option<String>,
    },
    Bip32 {
        mnemonic: Option<Mnemonic>,
    },
    Keypair {
        private_key: Secret,
    },
    MultiSig {
        mnemonics: Vec<Mnemonic>,
    },
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
enum Focus {
    #[default]
    None,
    AccountName,
    WalletSecret,
    PaymentSecret,
}

#[derive(Clone, Default)]
// #[derive(Default)]
struct Context {
    prv_key_data_info : Option<Arc<PrvKeyDataInfo>>,
    account_kind: Option<CreateAccountKind>,
    _create_private_key: bool,
    account_name: String,
    // enable_payment_secret: bool,
    wallet_secret : String,
    payment_secret: String,
    // payment_secret_confirm: String,
}

pub struct AccountCreate {
    #[allow(dead_code)]
    runtime: Runtime,
    // secret: String,
    context: Context,
    pub state: State,
    focus : FocusManager<Focus>,
}

impl AccountCreate {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            // secret: String::new(),
            state: State::Start,
            focus: FocusManager::default(),
            context: Default::default(),
        }
    }
}

impl ModuleT for AccountCreate {

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
        match self.state.clone() {
            State::Start => {

                let prv_key_data_map = core.prv_key_data_map.clone();

                Panel::new(self)
                    .with_caption("Create Account")
                    .with_back_enabled(core.has_stack(), |_this| {
                        core.back();
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        // ui.add_space(64.);
                        ui.label(i18n("Please select an account type"));
                        ui.label(" ");
                    })
                    .with_body(|this,ui|{

                        let margin = ui.available_width() * 0.5;

                        if let Some(prv_key_data_map) = prv_key_data_map {

                            for (_, prv_key_data_info) in prv_key_data_map.into_iter() {
                                ui.add(Separator::default().horizontal().shrink(margin));
                                ui.add_space(16.);
                                ui.label(format!("Private Key: {}", prv_key_data_info.name_or_id()));
                                ui.add_space(16.);
                                if ui.add(CompositeButton::new(
                                    "Kaspa Core HD account",
                                    "BIP-44 "
                                ))
                                .clicked() {
                                    this.context.prv_key_data_info = Some(prv_key_data_info.clone());
                                    this.context.account_kind = Some(CreateAccountKind::Bip44);
                                    this.state = State::AccountName;
                                    this.focus.next(Focus::AccountName);
                                }

                                ui.add_space(16.);
                            }
                        } else {
                            ui.label(i18n("No private keys found"));
                        }

                        ui.add(Separator::default().horizontal().shrink(margin));
                        ui.add_space(16.);

                    })
                    .render(ui);
            }
            State::AccountName => {

                Panel::new(self)
                    .with_caption(i18n("Account Name"))
                    .with_back(|this| {
                        this.state = State::Start;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.add_space(64.);
                        ui.label(i18n("Please enter the account name"));
                    })
                    .with_body(|this,ui| {

                        TextEditor::new(
                            &mut this.context.account_name,
                            &mut this.focus,
                            Focus::AccountName,
                            |ui, text| {
                                // ui.add_space(8.);
                                ui.label(RichText::new(i18n("Enter account name (optional)")).size(12.).raised());
                                ui.add_sized(theme_style().panel_editor_size, TextEdit::singleline(text)
                                    .vertical_align(Align::Center))
                            },
                        ).submit(|_,focus| {
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

                let submit = Rc::new(RefCell::new(false));

                Panel::new(self)
                    .with_caption(i18n("Wallet Secret"))
                    .with_back(|this| {
                        this.state = State::AccountName;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.label(i18n("Please enter the wallet secret"));
                    })
                    .with_body(|this,ui| {
                        TextEditor::new(
                            &mut this.context.wallet_secret,
                            &mut this.focus,
                            Focus::WalletSecret,
                            |ui, text| {
                                ui.label(RichText::new(i18n("Enter your wallet secret")).size(12.).raised());
                                ui.add_sized(theme_style().panel_editor_size, TextEdit::singleline(text)
                                    .vertical_align(Align::Center)
                                    .password(true))
                            },
                        ).submit(|text,_focus| {
                            if !text.is_empty() {
                                *submit.borrow_mut() = true;
                            }
                        })
                        .build(ui);
                    })
                    .with_footer(|this,ui| {
                        let enabled = !this.context.wallet_secret.is_empty();
                        if ui.large_button_enabled(enabled,i18n("Continue")).clicked() {
                            *submit.borrow_mut() = true;
                        }
                    })
                    .render(ui);

                if *submit.borrow() {
                    if self.context.prv_key_data_info.as_ref().map(|info| info.requires_bip39_passphrase()).unwrap_or(false) {
                        self.state = State::PaymentSecret;
                        self.focus.next(Focus::PaymentSecret);
                    } else {
                        self.state = State::CreateAccount;
                    }
                }
            }

            State::PaymentSecret => {
                Panel::new(self)
                    .with_caption(i18n("BIP-39 Passphrase"))
                    .with_back(|this| {
                        this.state = State::WalletSecret;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.label(i18n(i18n("Your private key requires BIP39 passphrase, please enter it now.")));
                    })
                    .with_body(|this,ui| {
                        TextEditor::new(
                            &mut this.context.payment_secret,
                            &mut this.focus,
                            Focus::PaymentSecret,
                            |ui, text| {
                                ui.label(RichText::new(i18n("Enter your BIP39 passphrase")).size(12.).raised());
                                ui.add_sized(theme_style().panel_editor_size, TextEdit::singleline(text)
                                    .vertical_align(Align::Center)
                                    .password(true))
                            },
                        ).submit(|text,focus| {
                            if !text.is_empty() {
                                this.state = State::CreateAccount;
                                focus.clear()
                            }
                        })
                        .build(ui);
                    })
                    .with_footer(|this,ui| {
                        let enabled = !this.context.payment_secret.is_empty();
                        if ui.large_button_enabled(enabled,i18n("Continue")).clicked() {
                            this.state = State::CreateAccount;
                        }
                    })
                    .render(ui);
            }

            State::PrivateKeyCreate => {
            }

            State::PrivateKeyConfirm => {
            }

            State::CreateAccount => {

                Panel::new(self)
                .with_caption(i18n("Creating Account"))
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
                self.context = Default::default();

                let account_create_result = Payload::<Result<AccountDescriptor>>::new("account_create_result");
                if !account_create_result.is_pending() {

                    let wallet = self.runtime.wallet().clone();
                    spawn_with_result(&account_create_result, async move {

                        let account_name = args.account_name.trim();
                        let account_name = (!account_name.is_empty()).then_some(account_name.to_string());
                        let wallet_secret = Secret::from(args.wallet_secret);
                        let payment_secret = args.prv_key_data_info.as_ref().and_then(|secret| {
                            secret.requires_bip39_passphrase().then_some(Secret::from(args.payment_secret))
                        });

                        let prv_key_data_id = *args.prv_key_data_info.as_ref().unwrap().id();

                        let prv_key_data_args = PrvKeyDataArgs { prv_key_data_id, payment_secret };
                        let account_args = AccountCreateArgsBip32 { account_name, account_index: None };
                        let account_create_args = AccountCreateArgs::Bip32 { prv_key_data_args, account_args };

                        let account_descriptor = wallet.accounts_create(wallet_secret, account_create_args).await?;
                        Ok(account_descriptor)
                    });
                }

                if let Some(result) = account_create_result.take() {
                    match result {
                        Ok(account_descriptor) => {
                            core.handle_account_creation(vec![account_descriptor]);
                            core.select::<modules::AccountManager>();
                            self.state = State::Start;
                        }
                        Err(err) => {
                            println!("Account creation error: {}", err);
                            self.state = State::AccountError(Arc::new(err));
                        }
                    }
                }
            }

            State::AccountError(err) => {
                Panel::new(self)
                .with_caption("Error")
                .with_header(move |this,ui| {
                    ui.label(" ");
                    ui.label(" ");
                    ui.label(RichText::new(i18n("Error creating account")).color(egui::Color32::from_rgb(255, 120, 120)));
                    ui.label(RichText::new(err.to_string()).color(egui::Color32::from_rgb(255, 120, 120)));

                    if ui.large_button(i18n("Restart")).clicked() {
                        this.state = State::Start;
                    }
                })
                .render(ui);
            }

            State::PresentMnemonic(_creation_data) => {
                unimplemented!();
            }

            State::ConfirmMnemonic(_creation_data) => {
                unimplemented!();
            }

            State::Finish(_account) => {
                unimplemented!();
            }

        }

    }
}
