#![allow(unused_imports)]

use crate::imports::*;
use kaspa_bip32::Mnemonic;
use kaspa_wallet_core::api::AccountsCreateRequest;
use kaspa_wallet_core::runtime::{AccountCreateArgs, PrvKeyDataCreateArgs, WalletCreateArgs};
use kaspa_wallet_core::storage::interface::AccessContext;
use kaspa_wallet_core::storage::{AccessContextT, AccountKind, PrvKeyDataInfo};


#[derive(Clone)]
pub enum CreateAccountKind {
    Bip44, // Create a BIP44 account on an existing private key
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
    Finish(Arc<dyn KaspaAccount>),
}

#[derive(Clone)]
pub enum CreationData {
    Bip44 {
        name : Option<String>,
    },
    Bip32 {
        mnemonic: Option<Mnemonic>,
        // account: Arc<dyn KaspaAccount>,
    },
    Keypair {
        private_key: Secret,
        // account: Arc<dyn KaspaAccount>,
    },
    MultiSig {
        mnemonics: Vec<Mnemonic>,
        // account: Arc<dyn KaspaAccount>,
    },
}

// impl CreationData {
//     pub fn account(&self) -> Arc<dyn KaspaAccount> {
//         match self {
//             Self::Bip32 { account, .. } => account.clone(),
//             Self::Keypair { account, .. } => account.clone(),
//             Self::MultiSig { account, .. } => account.clone(),
//         }
//     }
// }

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
        // ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {

            // let editor_size = egui::Vec2::new(200_f32, 40_f32);

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
                            ui.label("Please select an account type");
                            ui.label(" ");
                        })
                        .with_body(|this,ui|{

                            let margin = ui.available_width() * 0.5;

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
                                }

                                ui.add_space(16.);
                            }

                            ui.add(Separator::default().horizontal().shrink(margin));
                            ui.add_space(16.);

                        })
                        .render(ui);
                }
                State::AccountName => {
println!("rendering account name...");

                    Panel::new(self)
                        .with_caption("Account Name")
                        .with_back(|this| {
                            this.state = State::Start;
                        })
                        .with_close_enabled(false, |_|{
                        })
                        .with_header(|_ctx,ui| {
                            ui.add_space(64.);
                            ui.label("Please enter the account name");
                        })
                        .with_body(|this,ui| {

                            TextEditor::new(
                                &mut this.context.account_name,
                                &mut this.focus,
                                Focus::AccountName,
                                |ui, text| {
                                    // ui.add_space(8.);
                                    ui.label(RichText::new("Enter account name (optional)").size(12.).raised());
                                    ui.add_sized(theme_style().panel_editor_size, TextEdit::singleline(text)
                                        .vertical_align(Align::Center))
                                },
                            ).submit(|_,focus| {
                                println!("submit called...");
                                this.state = State::WalletSecret;
                                focus.next(Focus::WalletSecret);
                            })
                            .build(ui);
                    
                        })
                        .with_footer(|this,ui| {
                            let size = theme_style().large_button_size;
                            if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                                this.state = State::WalletSecret;
                                this.focus.next(Focus::WalletSecret);
                            }
                        })
                        .render(ui);
                }


                State::WalletSecret => {

                    let mut submit_via_editor = false;
                    let mut submit_via_footer = false;

                    Panel::new(self)
                        .with_caption("Wallet Secret")
                        .with_back(|this| {
                            this.state = State::AccountName;
                        })
                        .with_close_enabled(false, |_|{
                        })
                        .with_header(|_ctx,ui| {
                            ui.label("Please enter the wallet secret");
                        })
                        .with_body(|this,ui| {
                            TextEditor::new(
                                &mut this.context.wallet_secret,
                                &mut this.focus,
                                Focus::WalletSecret,
                                |ui, text| {
                                    ui.label(egui::RichText::new("Enter your wallet secret").size(12.).raised());
                                    ui.add_sized(theme_style().panel_editor_size, TextEdit::singleline(text)
                                        .vertical_align(Align::Center)
                                        .password(true))
                                },
                            ).submit(|text,_focus| {
                                if !text.is_empty() {
                                    submit_via_editor = true;
                                }
                            })
                            .build(ui);
                        })
                        .with_footer(|this,ui| {
                            let size = theme_style().large_button_size;
                            let enabled = !this.context.wallet_secret.is_empty();
                            if ui.add_enabled(enabled, egui::Button::new("Continue").min_size(size)).clicked() {
                                submit_via_footer = true;
                            }
                        })
                        .render(ui);

                    if submit_via_editor || submit_via_footer {
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
                        .with_caption("BIP-39 Passphrase")
                        .with_back(|this| {
                            this.state = State::WalletSecret;
                        })
                        .with_close_enabled(false, |_|{
                        })
                        .with_header(|_ctx,ui| {
                            ui.label(i18n("Your private key requires BIP39 passphrase, please enter it now."));
                        })
                        .with_body(|this,_ui| {
                            TextEditor::new(
                                &mut this.context.payment_secret,
                                &mut this.focus,
                                Focus::PaymentSecret,
                                |ui, text| {
                                    ui.label(egui::RichText::new("Enter your BIP39 passphrase").size(12.).raised());
                                    ui.add_sized(theme_style().panel_editor_size, TextEdit::singleline(text)
                                        .vertical_align(Align::Center)
                                        .password(true))
                                },
                            ).submit(|text,focus| {
                                if !text.is_empty() {
                                    this.state = State::CreateAccount;
                                    focus.clear()
                                }
                            });
                        })
                        .with_footer(|this,ui| {
                            let size = theme_style().large_button_size;
                            let enabled = !this.context.payment_secret.is_empty();
                            if ui.add_enabled(enabled, Button::new("Continue").min_size(size)).clicked() {
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
                    .with_caption("Creating Account")
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
                    self.context = Default::default();
                    // let args = &self.context;
                    // let account_name = self.context.account_name.trim().clone();
                    // let account_name = (!account_name.is_empty()).then_some(account_name.to_string());
                    // let account_kind = AccountKind::Bip32;
                    // let wallet_secret = Secret::from(self.context.wallet_secret.clone());
                    // let payment_secret = self.context.prv_key_data_info.as_ref().and_then(|secret| {
                    //     secret.requires_bip39_passphrase().then_some(Secret::from(self.context.payment_secret))
                    // });

                    // let prv_key_data_id = self.context.prv_key_data_info.as_ref().unwrap().id();


                    // let prv_key_data_info = args.prv_key_data_info.clone().unwrap();
                    let account_create_result = Payload::<Result<AccountDescriptor>>::new("wallet_create_result");
                    if !account_create_result.is_pending() {

                        let wallet = self.runtime.wallet().clone();
                        spawn_with_result(&account_create_result, async move {

                            let account_name = args.account_name.trim();
                            let account_name = (!account_name.is_empty()).then_some(account_name.to_string());
                            let account_kind = AccountKind::Bip32;
                            let wallet_secret = Secret::from(args.wallet_secret);
                            let payment_secret = args.prv_key_data_info.as_ref().and_then(|secret| {
                                secret.requires_bip39_passphrase().then_some(Secret::from(args.payment_secret))
                            });

                            let prv_key_data_id = args.prv_key_data_info.as_ref().unwrap().id();

                            let account_args = AccountCreateArgs {
                                account_name,
                                account_kind,
                                wallet_secret,
                                payment_secret,
                            };

                            Ok(wallet.accounts_create(*prv_key_data_id, account_args).await?)
                        });
                    }

                    if let Some(result) = account_create_result.take() {
                        match result {
                            Ok(descriptor) => {
                                println!("Account created successfully");
                                // let account = Account::from(descriptor);
                                // core.account_collection.as_mut().expect("account collection").push_unchecked(account.clone());
                                // core.get_mut::<modules::AccountManager>().select(Some(account));
                                // core.select::<modules::AccountManager>();

                                // - RESET STATE
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
                        ui.label(egui::RichText::new("Error creating account").color(egui::Color32::from_rgb(255, 120, 120)));
                        ui.label(egui::RichText::new(err.to_string()).color(egui::Color32::from_rgb(255, 120, 120)));

                        if ui.add_sized(theme_style().panel_editor_size, egui::Button::new("Restart")).clicked() {
                            this.state = State::Start;
                        }
                    })
                    .render(ui);
                }

                State::PresentMnemonic(_creation_data) => {
                    unimplemented!();
                    // let mut phrase = creation_data.mnemonic.phrase().to_string();

                    // Panel::new(self)
                    //     .with_caption("Private Key Mnemonic")
                    //     .with_body(|_this,ui| {
                    //         ui.label(RichText::new("Your mnemonic phrase allows your to re-create your private key. \
                    //             The person who has access to this mnemonic will have full control of \
                    //             the Kaspa stored in it. Keep your mnemonic safe. Write it down and \
                    //             store it in a safe, preferably in a fire-resistant location. Do not \
                    //             store your mnemonic on this computer or a mobile device. This wallet \
                    //             will never ask you for this mnemonic phrase unless you manually \
                    //             initiate a private key recovery.").size(14.));
                    //         ui.label(" ");
                    //         ui.label(RichText::new("Never share your mnemonic with anyone!").color(Color32::RED));
                    //         ui.label(" ");
                    //         ui.label("Your default account private key mnemonic is:");
                    //         ui.label(" ");
                    //         ui.separator();
                    //         ui.label(" ");

                    //         let words = phrase.split(' ').collect::<Vec<&str>>();
                    //         let chunks = words.chunks(6).collect::<Vec<&[&str]>>();
                    //         for chunk in chunks {
                    //             ui.horizontal(|ui| {
                    //                 ui.columns(6, |cols| {

                    //                     for col in 0..chunk.len() {
                    //                         cols[col].label(egui::RichText::new(chunk[col]).family(FontFamily::Monospace).size(14.).color(egui::Color32::WHITE));
                    //                     }
                    //                 })
                    //             });
                    //         }

                    //         phrase.zeroize();

                    // })
                    // .with_footer(|this,ui| {
                    //     if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                    //         this.state = State::ConfirmMnemonic(mnemonic);
                    //     }
                    // })
                    // .render(ui);

                }

                State::ConfirmMnemonic(_creation_data) => {
                    unimplemented!();
                    // let creation_data_back = creation_data.clone();
                    // Panel::new(self)
                    //     .with_caption("Confirm Mnemonic")
                    //     .with_back(move |this|{
                    //         this.state = State::PresentMnemonic(creation_data_back);
                    //     })
                    //     .with_header(|_this,ui| {
                    //         ui.label("Please validate your mnemonic");
                    //     })
                    //     .with_footer(move |this,ui| {
                    //         if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                    //             this.state = State::Finish(creation_data.account());
                    //         }
                    //     })
                    //     .render(ui);
                }

                State::Finish(_account) => {
                    unimplemented!();

                    // Panel::new(self)
                    //     .with_caption("Account Created")
                    //     .with_body(|_this,ui| {
                    //         ui.label(" ");
                    //         ui.label("Your account has been created and is ready to use.");
                    //         ui.label(" ");
                    //     })
                    //     .with_footer(move |this,ui| {
                    //         if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                    //             this.state = State::Start;

                    //             // TODO - add account to wallet ^^^
                    //             let descriptor = account.descriptor().unwrap();
                    //             let account = Account::from(descriptor);
                    //             core.account_collection.as_mut().unwrap().push(account.clone());

                    //             core.select::<modules::AccountManager>();
                    //             core.get_mut::<modules::AccountManager>().select(Some(account));
                    //         }
                    //     })
                    //     .render(ui);
                }

            }

        // });
    }
}
