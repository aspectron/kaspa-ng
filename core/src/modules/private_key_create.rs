#![allow(unused_imports)]

use crate::imports::*;
use kaspa_bip32::Mnemonic;
// use kaspa_wallet_core::runtime::{PrvKeyDataCreateArgs, WalletCreateArgs};
// use kaspa_wallet_core::storage::AccountKind;

pub enum MnemonicSize {
    Words12,
    Words24,
}

pub enum CreateAccountDescriptor {
    Bip44, // derivation for the same private key
    Bip32,
    Legacy,
    MultiSig,
    Keypair,
    // Keypair,
    // MultiSig,
}

impl CreateAccountDescriptor {
    fn _describe(&self) -> (&'static str, &'static str) {
        match self {
            Self::Bip44 => ("BIP44", "BIP44"),
            Self::Bip32 => ("BIP32", "BIP32"),
            Self::Legacy => ("Legacy", "Legacy"),
            Self::MultiSig => ("MultiSig", "MultiSig"),
            Self::Keypair => ("Keypair", "Keypair"),
        }
    }
}



#[derive(Clone)]
pub enum State {
    Start,
    PrivateKeyCreate,
    PrivateKeyConfirm,
    AccountName,
    PaymentSecret,
    CreateAccount,
    AccountError(Arc<Error>),
    PresentMnemonic(Arc<CreationData>),
    ConfirmMnemonic(Arc<CreationData>),
    Finish(Arc<dyn CoreAccount>),
}

pub enum CreationData {
    Bip32 {
        mnemonic: Option<Mnemonic>,
        account: Arc<dyn CoreAccount>,
    },
    Keypair {
        private_key: Secret,
        account: Arc<dyn CoreAccount>,
    },
    MultiSig {
        mnemonics: Vec<Mnemonic>,
        account: Arc<dyn CoreAccount>,
    },
}

impl CreationData {
    pub fn account(&self) -> Arc<dyn CoreAccount> {
        match self {
            Self::Bip32 { account, .. } => account.clone(),
            Self::Keypair { account, .. } => account.clone(),
            Self::MultiSig { account, .. } => account.clone(),
        }
    }
}

#[derive(Clone, Default)]
struct Context {
    _account_kind: Option<AccountKind>,
    _account_name: String,
    account_title: String,
    _create_private_key: bool,
    enable_payment_secret: bool,
    payment_secret: String,
    payment_secret_confirm: String,
}

pub struct PrivateKeyCreate {
    #[allow(dead_code)]
    runtime: Runtime,
    // secret: String,
    args: Context,
    pub state: State,
}

impl PrivateKeyCreate {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            // secret: String::new(),
            state: State::Start,
            args: Default::default(),
        }
    }
}

impl ModuleT for PrivateKeyCreate {

    fn modal(&self) -> bool { true }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {

            match self.state.clone() {
                State::Start => {


                    // let v= core.account_collection().map(|collection|{
                    //     HashMap::from_iter(collection.iter().map(|account|{
                    //         (account.descriptor().private_key_id(),account.clone())
                    //         // println!("account: {}", account.title());
                    //     }))
                    // }).unwrap_or_default();

                    // if let Some(account_collection) = core.account_collection() {
                    // }


                    Panel::new(self)
                        .with_caption("Create Private Key")
                        .with_close_enabled(false, |_|{
                        })
                        .with_header(|_ctx,ui| {
                            // ui.add_space(64.);
                            ui.label(i18n("Please select an account type"));
                            ui.label(" ");
                        })
                        .with_body(|_this,_ui|{

                            // for (prv_key_data_id, prv_key_data_info) in core.prv_key_data_map.iter() {
                            //     ui.label(format!("Private Key: {}", prv_key_data_info.name_or_id()));
                            //     ui.add_space(16.);
                            //     if ui.add(CompositeButton::new(
                            //         "HD account",
                            //         "BIP-44 "
                            //     ))
                            //     .clicked() {
                                        
                            //     }
                            // }

                            
                            // ui.label("A wallet is stored in a file on your computer. You can create multiple wallet.");
                        })
                        .with_footer(|_this,ui| {
                            // if ui.add_sized(theme().large_button_size, egui::Button::new(i18n("Continue"))).clicked() {
                            if ui.large_button(i18n("Continue")).clicked() {
                                // this.state = State::WalletName;
                            }
                        })
                        .render(ui);
                }
                State::AccountName => {

                    // TODO - check if wallet exists
                    let _wallet_exists_result = Payload::<Result<bool>>::new("wallet_exists_result");

                    Panel::new(self)
                    .with_caption(i18n("Account Name"))
                    .with_back(|this| {
                        this.state = State::Start;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.add_space(64.);
                        ui.label(i18n("Please specify the name of the wallet"));
                    })
                    .with_body(|this,ui| {
                        ui.add_sized(
                            theme_style().panel_editor_size,
                            TextEdit::singleline(&mut this.args.account_title)
                                .hint_text(i18n("Account Name..."))
                                .vertical_align(Align::Center),
                        );
                    })
                    .with_footer(|this,ui| {
                        if ui.large_button(i18n("Continue")).clicked() {
                            this.state = State::AccountName;
                        }
                    })
                    .render(ui);
                }


                State::PaymentSecret => {

                    Panel::new(self)
                        .with_caption(i18n("Payment & Recovery Password"))
                        .with_back(|this| {
                            this.state = State::AccountName;
                        })
                        .with_close_enabled(false, |_|{
                        })
                        .with_header(|_ctx,ui| {
                            ui.heading(i18n("Optional"));
                            ui.label(" ");
                            ui.label(i18n("The optional payment & recovery password, if provided, will be required to \
                                send payments. This password will also be required when recovering your wallet \
                                in addition to your private key or mnemonic. If you lose this password, you will not \
                                be able to use mnemonic to recover your wallet!"));
                        })
                        .with_body(|this,ui| {
                            ui.label(RichText::new(i18n("ENTER YOUR PAYMENT PASSWORD")).size(12.).raised());
                            ui.add_sized(
                                theme_style().panel_editor_size,
                                TextEdit::singleline(&mut this.args.payment_secret)
                                    .hint_text(i18n("Payment password..."))
                                    .vertical_align(Align::Center),
                            );

                            ui.label(" ");
                            ui.label(RichText::new(i18n("VERIFY YOUR PAYMENT PASSWORD")).size(12.).raised());

                            ui.add_sized(
                                theme_style().panel_editor_size,
                                TextEdit::singleline(&mut this.args.payment_secret_confirm)
                                    .hint_text(i18n("Payment password..."))
                                    .vertical_align(Align::Center),
                            );

                            if this.args.payment_secret_confirm.is_not_empty() && this.args.payment_secret != this.args.payment_secret_confirm {
                                ui.label(" ");
                                ui.label(RichText::new("Passwords do not match").color(egui::Color32::from_rgb(255, 120, 120)));
                                ui.label(" ");
                            } else {
                                ui.label(" ");
                            }
                        })
                        .with_footer(|this,ui| {
                            let ok = this.args.payment_secret == this.args.payment_secret_confirm;
                            if ui.large_button_enabled(ok,i18n("Continue")).clicked() {
                                this.state = State::Start;
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

                    let args = self.args.clone();
                    let wallet_create_result = Payload::<Result<Arc<CreationData>>>::new("wallet_create_result");
                    if !wallet_create_result.is_pending() {

                        // TODO CREATE WALLET !
                        let _wallet = self.runtime.wallet().clone();
                        spawn_with_result(&wallet_create_result, async move {

                            if args.enable_payment_secret && args.payment_secret.is_empty() {
                                return Err(Error::custom(i18n("Payment secret is empty")));
                            }

                            // if args.enable_phishing_hint && args.phishing_hint.is_empty() {
                            //     return Err(Error::custom("Phishing hint is empty"));
                            // }

                            // let wallet_secret = Secret::from(args.wallet_secret);
                            // let payment_secret = args.enable_payment_secret.then_some(Secret::from(args.payment_secret));

                            // // suspend commits for multiple operations
                            // wallet.store().batch().await?;

                            // let account_kind = AccountKind::Bip32;
                            // let wallet_args = WalletCreateArgs::new(
                            //     args.wallet_title.is_not_empty().then_some(args.wallet_title),
                            //     args.wallet_filename.is_not_empty().then_some(args.wallet_filename),
                            //     args.enable_phishing_hint.then_some(args.phishing_hint.into()), 
                            //     wallet_secret.clone(),
                            //     false
                            // );
                            // let prv_key_data_args = PrvKeyDataCreateArgs::new(
                            //     None, 
                            //     wallet_secret.clone(), 
                            //     payment_secret.clone()
                            // );
                            // let account_args = PrivateKeyCreateArgs::new(
                            //     args.account_name.is_not_empty().then_some(args.account_name), 
                            //     args.account_title.is_not_empty().then_some(args.account_title), 
                            //     account_kind, 
                            //     wallet_secret.clone(), 
                            //     payment_secret.clone(),
                            // );
                            // let _descriptor = wallet.create_wallet(wallet_args).await?;
                            // let (prv_key_data_id, mnemonic) = wallet.create_prv_key_data(prv_key_data_args).await?;
                            // let account = wallet.create_bip32_account(prv_key_data_id, account_args).await?;

                            // // flush data to storage
                            // let access_ctx: Arc<dyn AccessContextT> = Arc::new(AccessContext::new(wallet_secret));
                            // wallet.store().flush(&access_ctx).await?;

                            // Ok((Arc::new(mnemonic), account))

                            unimplemented!()
                        });
                    }

                    if let Some(result) = wallet_create_result.take() {
                        match result {
                            Ok(creation_data) => {
                                // println!("Account created successfully");
                                self.state = State::PresentMnemonic(creation_data);
                                // wallet.get_mut::<section::Account>().select(Some(creation_dataaccount));
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
                    // let mut phrase = creation_data.mnemonic.phrase().to_string();

                    // Panel::new(self)
                    //     .with_caption("Private Key Mnemonic")
                    //     .with_body(|_this,ui| {
                    //         ui.label(RichText::new("Your mnemonic phrase allows you to re-create your private key. \
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
                    //                         cols[col].label(RichText::new(chunk[col]).family(FontFamily::Monospace).size(14.).color(egui::Color32::WHITE));
                    //                     }
                    //                 })
                    //             });
                    //         }

                    //         phrase.zeroize();

                    // })
                    // .with_footer(|this,ui| {
                    //     if ui.add_sized(size, egui::Button::new(i18n("Continue"))).clicked() {
                    //         this.state = State::ConfirmMnemonic(mnemonic);
                    //     }
                    // })
                    // .render(ui);

                }

                State::ConfirmMnemonic(creation_data) => {
                    let creation_data_back = creation_data.clone();
                    Panel::new(self)
                        .with_caption(i18n("Confirm Mnemonic"))
                        .with_back(move |this|{
                            this.state = State::PresentMnemonic(creation_data_back);
                        })
                        .with_header(|_this,ui| {
                            ui.label(i18n("Please validate your mnemonic"));
                        })
                        .with_footer(move |this,ui| {
                            if ui.large_button(i18n("Continue")).clicked() {
                                this.state = State::Finish(creation_data.account());
                            }
                        })
                        .render(ui);
                }

                State::Finish(account) => {

                    Panel::new(self)
                        .with_caption(i18n("Account Created"))
                        .with_body(|_this,ui| {
                            ui.label(" ");
                            ui.label(i18n("Your account has been created and is ready to use."));
                            ui.label(" ");
                        })
                        .with_footer(move |this,ui| {
                            if ui.large_button(i18n("Continue")).clicked() {
                                this.state = State::Start;

                                // TODO - add account to wallet ^^^
                                let descriptor = account.descriptor().unwrap();
                                let account = Account::from(core.network(), descriptor);
                                core.account_collection.as_mut().unwrap().push_unchecked(account.clone());

                                core.select::<modules::AccountManager>();
                                let device = core.device().clone();
                                let wallet = core.wallet();
                                core.get_mut::<modules::AccountManager>().select(wallet, Some(account), device, true);
                            }
                        })
                        .render(ui);
                }

            }

        });
    }
}
