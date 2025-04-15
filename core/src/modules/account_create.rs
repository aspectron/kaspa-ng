use kaspa_wallet_core::storage::keydata::PrvKeyDataVariantKind;
use modules::wallet_create::WalletCreate;

use crate::imports::*;
use kaspa_wallet_core::storage::AssocPrvKeyDataIds;
use kaspa_wallet_core::deterministic::bip32::BIP32_ACCOUNT_KIND;

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
    KeySelection,
    StartImport,
    ImportMnemonic,
    ImportMnemonicWithEditor,
    PrivateKeyCreate,
    PrivateKeyConfirm,
    AccountName,
    WalletSecret,
    PaymentSecret,
    AddAccount,
    //CreateAccount,
    //ImportAccount,
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
    WalletMnemonic,
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
    word_count: WordCount,
    import_mnemonic: bool,
    import_legacy: bool,
    import_with_bip39_passphrase: bool,
    import_private_key_mnemonic: String,
    prv_keys: Vec<Arc<PrvKeyDataInfo>>,
}

impl Zeroize for Context {
    fn zeroize(&mut self) {
        self.account_name.zeroize();
        self.wallet_secret.zeroize();
        self.payment_secret.zeroize();
        self.import_private_key_mnemonic.zeroize();
    }
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
                {
                    let account_list = core.account_collection().iter();
                    let mut prv_keys = HashMap::new();
                    if let Some(prv_key_data_map) = core.prv_key_data_map.clone(){
                        account_list
                        .flat_map(|a|a.iter())
                        .for_each(|account|{
                            if account.account_kind() != &BIP32_ACCOUNT_KIND {
                                return;
                            }
                            if let AssocPrvKeyDataIds::Single(key_id) = account.descriptor().prv_key_data_ids {
                                if let Some(prv_key) = prv_key_data_map.get(&key_id){
                                    prv_keys.insert(*prv_key.id(), prv_key.clone());
                                }
                            }
                        });
                    }
                    self.context.prv_keys = prv_keys.into_values().collect();
                }
                
                Panel::new(self)
                    .with_caption("Add Account")
                    .with_back_enabled(core.has_stack(), |this| {
                        this.context.zeroize();
                        core.back();
                    })
                    .with_close_enabled(false, |_|{ })
                    .with_header(|_ctx,ui| {
                        // ui.add_space(64.);
                        ui.label(i18n("Add a new account by importing a mnemonic"));
                        ui.label(i18n("or by deriving an already imported BIP-44 private key."));
                        ui.label(" ");
                    })
                    .with_body(|this,ui|{
                        let no_keys = this.context.prv_keys.is_empty();
                        if ui.large_button_enabled(!no_keys, i18n("Create Account")).clicked(){
                            this.state = State::KeySelection;
                        }
                        ui.label(i18n("Derive a private key to create a new account."));
                        if no_keys {
                            ui.label(i18n("No BIP-44 private keys found"));
                        }

                        ui.add(ui.create_separator(Some(32.0), 0.5, Some(true)));


                        if ui.large_button(i18n("Import Account")).clicked(){
                            this.state = State::StartImport;
                        }
                        ui.label(i18n("Create an account by importing a private key."));

                    })
                    .render(ui);
            }
            State::KeySelection => {
                self.context.import_mnemonic = false;
                Panel::new(self)
                    .with_caption("Create Account")
                    .with_back_enabled(core.has_stack(), |this| {
                        this.state = State::Start;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        // ui.add_space(64.);
                        ui.label(i18n("Please select private key"));
                        ui.label(" ");
                    })
                    .with_body(|this,ui|{

                        let margin = ui.available_width() * 0.5;

                        if !this.context.prv_keys.is_empty() {

                            for prv_key_data_info in &this.context.prv_keys{
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
            State::StartImport => {
                self.context.import_mnemonic = true;
                self.context.import_private_key_mnemonic.zeroize();
                self.context.wallet_secret.zeroize();
                self.context.payment_secret.zeroize();
                self.context.prv_key_data_info = None;

                let submit = WalletCreate::import_selection::<State>(
                    &mut self.state,
                    &mut self.context.word_count,
                    &mut self.context.import_legacy,
                    &mut self.context.import_with_bip39_passphrase,
                    ui,
                    Some(|state: &mut State|{
                        *state = State::Start;
                    }
                ));
                if submit {
                    self.state = State::ImportMnemonic;
                }
            }

            State::ImportMnemonic => {
                self.state = State::ImportMnemonicWithEditor;
                self.focus.next(Focus::WalletMnemonic);
            }

            State::ImportMnemonicWithEditor => {
                let proceed = WalletCreate::import_mnemonic::<Focus, State>(
                    &mut self.state,
                    &mut self.context.import_private_key_mnemonic,
                    &self.context.word_count,
                    &mut self.focus,
                    Focus::WalletMnemonic,
                    ui,
                    |m| {
                        *m = State::StartImport;
                    }
                );

                if proceed {
                    self.state = State::AccountName;
                    self.focus.clear();
                }

            }

            State::AccountName => {

                Panel::new(self)
                    .with_caption(i18n("Account Name"))
                    .with_back(|this| {
                        if this.context.import_mnemonic{
                            this.state = State::StartImport;
                        }else{
                            this.state = State::Start;
                        }
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
                    if self.context.import_with_bip39_passphrase || self.context.prv_key_data_info.as_ref().map(|info| info.requires_bip39_passphrase()).unwrap_or(false) {
                        self.state = State::PaymentSecret;
                        self.focus.next(Focus::PaymentSecret);
                    } else {
                        self.state = State::AddAccount;
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
                        ui.label(i18n("Your private key requires BIP39 passphrase, please enter it now."));
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
                                this.state = State::AddAccount;
                                focus.clear()
                            }
                        })
                        .build(ui);
                    })
                    .with_footer(|this,ui| {
                        let enabled = !this.context.payment_secret.is_empty();
                        if ui.large_button_enabled(enabled,i18n("Continue")).clicked() {
                            this.state = State::AddAccount;
                        }
                    })
                    .render(ui);
            }

            State::PrivateKeyCreate => {
            }

            State::PrivateKeyConfirm => {
            }

            State::AddAccount => {
                let caption = if self.context.import_mnemonic {i18n("Importing Account")}else{i18n("Creating Account")};

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

                let args = self.context.clone();
                self.context = Default::default();

                let account_create_result = Payload::<Result<AccountDescriptor>>::new("account_create_result");
                if !account_create_result.is_pending() {

                    let wallet = self.runtime.wallet().clone();
                    spawn_with_result(&account_create_result, async move {
                        sleep(Duration::from_secs(2)).await;
                        let account_name = args.account_name.trim();
                        let account_name = account_name.is_not_empty().then_some(account_name.to_string());
                        let wallet_secret = Secret::from(args.wallet_secret);

                        let payment_secret;

                        let prv_key_data_id = if args.import_mnemonic {
                            let requires_bip39_passphrase = !args.import_legacy && args.import_with_bip39_passphrase;
                            payment_secret = requires_bip39_passphrase.then_some(Secret::from(args.payment_secret.as_str()));
                            let mnemonic = Secret::from(sanitize_mnemonic(args.import_private_key_mnemonic.as_str()));
                            let key_data_name = None;
                            let prv_key_data_args = PrvKeyDataCreateArgs::new(
                                key_data_name,
                                payment_secret.clone(),
                                mnemonic,
                                PrvKeyDataVariantKind::Mnemonic,
                            );
                            wallet.clone().prv_key_data_create(wallet_secret.clone(), prv_key_data_args).await?
                        }else{
                            payment_secret = args.prv_key_data_info.as_ref().and_then(|key| {
                                key.requires_bip39_passphrase().then_some(Secret::from(args.payment_secret))
                            });
                            *args.prv_key_data_info.as_ref().unwrap().id()
                        };

                        let account_create_args = if args.import_legacy {
                            AccountCreateArgs::new_legacy(prv_key_data_id, account_name)
                        }else{
                            AccountCreateArgs::new_bip32(prv_key_data_id, payment_secret, account_name, None)
                        };
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
                            if self.context.import_mnemonic{
                                log_info!("Account import error: {}", err);
                            }else{
                                log_info!("Account creation error: {}", err);
                            }
                            
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
                        if this.context.import_mnemonic{
                            this.state = State::StartImport;
                        }else{
                            this.state = State::Start;
                        }
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
