#![allow(unused_imports)]

use crate::imports::*;
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
struct Context {
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

pub struct CreateWallet {
    #[allow(dead_code)]
    interop: Interop,
    secret: String,
    args : Context,
    pub state: State,
    // pub message: Option<String>,
    // selected_wallet : Option<String>,
}

impl CreateWallet {
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
            let wallet_exists_result = Payload::<Result<bool>>::new("wallet_exists_result");

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
                        // .with_body(|this,ui| {
                        //     for wallet in wallet.wallet_list.iter() {
                        //         if ui.add_sized(size, egui::Button::new(wallet.filename.clone())).clicked() {
                        //             this.selected_wallet = Some(wallet.filename.clone());
                        //             this.state = State::Unlock(None);
                        //         }
                        //     }
                        //     ui.label(" ");
                        //     ui.separator();
                        //     ui.label(" ");
                        //     if ui.add_sized(size, egui::Button::new("Create new wallet")).clicked() {
                        //         wallet.select::<section::CreateWallet>();
                        //     }

                        //     ui.label(" ");
                        // })
                        .with_footer(|this,ui| {
                            // if ui.add_sized(theme().large_button_size, egui::Button::new("Continue")).clicked() {
                            let size = theme().large_button_size;
                            if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                                this.state = State::WalletName;
                            }
                        })
                        .render(ui);

                    // ui.label(" ");
                    // ui.heading("Create a Wallet");
                    // ui.label(" ");
                    // ui.label(" ");
                    // ui.label("The following will guide you through the process of creating a new wallet");
                    // ui.label(" ");
                    // ui.label("A wallet is stored in a file on your computer. You can create multiple wallet.");
                    // ui.label(" ");

                    // ui.add_space(32.);

                    // egui::ScrollArea::vertical()
                    //     // .id_source("wallet-list")
                        
                    //     .show(ui, |ui| {
                    //         ui.set_width(ui.available_width());
                    //         ui.set_height(ui.available_height()-64.);

                    //     });
                    // if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                    //     self.state = State::WalletName;
                    // }
                    // ui.add_space(64.);
        
                }
                State::WalletName => {
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

                            if this.args.wallet_secret_confirm.len() > 0 && this.args.wallet_secret != this.args.wallet_secret_confirm {
                                ui.label(" ");
                                ui.label(egui::RichText::new("Passwords do not match").color(egui::Color32::from_rgb(255, 120, 120)));
                                ui.label(" ");
                            } else {
                                ui.label(" ");
                                // ui.label(" ");
                                // ui.label(" ");
                            }
                        })
                        .with_footer(|this,ui| {
                            let size = theme().large_button_size;
                            let ok = this.args.wallet_secret == this.args.wallet_secret_confirm && this.args.wallet_secret.len() > 0;
                            if ui.add_enabled(ok, egui::Button::new("Continue").min_size(size)).clicked() {
                                this.state = State::PaymentSecret;
                            }
                        })
                        .render(ui);
        

                    // ----

                    // ui.heading("Wallet Encryption Password");
                    // ui.label(" ");
                    // ui.label("Wallet password is used to encrypt your wallet.");
                    // ui.label(" ");

                    // ui.add_space(32.);

                    // egui::ScrollArea::vertical()
                    //     .show(ui, |ui| {
                    //         ui.set_width(ui.available_width());



                    //         // if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                    //         //     self.state = State::AccountName;
                    //         // }
                    //     });
        
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

                            if this.args.wallet_secret_confirm.len() > 0 && this.args.payment_secret != this.args.payment_secret_confirm {
                                ui.label(" ");
                                ui.label(egui::RichText::new("Passwords do not match").color(egui::Color32::from_rgb(255, 120, 120)));
                                ui.label(" ");
                            } else {
                                ui.label(" ");
                                // ui.label(" ");
                                // ui.label(" ");
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
        


                    // ---

                    // ui.heading("Payment & Recovery Password");
                    // ui.label(" ");
                    // ui.heading("Optional");
                    // ui.label(" ");
                    
                    // egui::ScrollArea::vertical()
                    //     .show(ui, |ui| {
                    //         ui.set_width(ui.available_width());

                    //         ui.label("The optional payment & recovery password, if provided, will be required to \
                    //         send payments. This password will also be required when recovering your wallet \
                    //         in addition to your private key or mnemonic. If you loose this password, you will not \
                    //         be able to use mnemonic to recover your wallet!");
                    //         ui.label(" ");
        
                    //         ui.add_space(32.);

                    //         ui.add_sized(
                    //             size,
                    //             TextEdit::singleline(&mut self.args.payment_secret)
                    //                 .hint_text("Payment password...")
                    //                 .vertical_align(Align::Center),
                    //         );

                    //         ui.add_sized(
                    //             size,
                    //             TextEdit::singleline(&mut self.args.payment_secret_confirm)
                    //                 .hint_text("Payment password...")
                    //                 .vertical_align(Align::Center),
                    //         );

                    //         let ok = self.args.wallet_secret == self.args.wallet_secret_confirm;
                    //         if ui.add_enabled(ok, egui::Button::new("Continue").min_size(size)).clicked() {
                    //             self.state = State::CreateWallet;
                    //         }
                    //         // if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                    //         //     self.state = State::AccountName;
                    //         // }
                    //     });
        
                }
                State::CreateWallet => {

                    Panel::new(self)
                    .with_caption("Creating Wallet")
                    .with_header(|this, ui|{
                        ui.label(" ");
                        ui.label("Please wait...");
                        ui.label(" ");
                        ui.label(" ");
                        ui.add_space(64.);
                        ui.add(egui::Spinner::new().size(92.));
                    })
                    .render(ui);

                    // ui.heading("Creating Wallet");
                    // ui.heading("Please wait...");
                    // ui.add_space(64.);

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

                    panel(self)
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
                
                // ui.heading("Error");
                //     ui.label(" ");

                //     if ui.add_sized(size, egui::Button::new("Restart")).clicked() {
                //         self.state = State::Start;
                //     }
                }

                State::PresentMnemonic(mnemonic) => {

                    panel(self)
                    .with_caption("Private Key Mnemonic")
                    .with_body(move |_this,ui| {
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

                        let phrase = mnemonic.phrase();
                        let words = phrase.split(" ").collect::<Vec<&str>>();
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

                        // ui.label(egui::RichText::new(text).family(FontFamily::Monospace).color(egui::Color32::WHITE));
                        // ui.label(egui::RichText::new(mnemonic.phrase()).family(FontFamily::Monospace).color(egui::Color32::WHITE));

                        // ui.add(TextEdit::multiline(&mut this.args.payment_secret_confirm)
                        // .vertical_align(Align::Center));
                    })
                    .with_footer(move |this,ui| {
                        if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                            this.state = State::ConfirmMnemonic;
                        }
                    })
                    .render(ui);

                    // ui.heading("Private Key Mnemonic");
                    // ui.label(" ");

                    // egui::ScrollArea::vertical()
                    // .show(ui, |ui| {
                    //     ui.set_width(ui.available_width());

                    //     ui.label("Your mnemonic phrase allows your to re-create your private key. \
                    //     The person who has access to this mnemonic will have full control of \
                    //     the Kaspa stored in it. Keep your mnemonic safe. Write it down and \
                    //     store it in a safe, preferably in a fire-resistant location. Do not \
                    //     store your mnemonic on this computer or a mobile device. This wallet \
                    //     will never ask you for this mnemonic phrase unless you manually \
                    //     initiate a private key recovery.");
                    //     ui.label(" ");
                    //     ui.label("Never share your mnemonic with anyone!");
                    //     ui.label(" ");
                    //     ui.label("Your default account private key mnemonic is:");
                    //     ui.separator();
                    //     ui.label(egui::RichText::new(mnemonic.phrase()).color(egui::Color32::WHITE));

                    //     ui.add(TextEdit::multiline(&mut self.args.payment_secret_confirm)
                    //     // .hint_text("Payment password...")
                        
                    //     .vertical_align(Align::Center));



                    //     ui.separator();
                    //     ui.label(" ");

                    //     if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                    //         self.state = State::ConfirmMnemonic;
                    //     }

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
                    // });
                }

                State::ConfirmMnemonic => {
                    panel(self)
                    .with_caption("Confirm Mnemonic")
                    .with_header(|_this,ui| {

                    })
                    .with_footer(move |this,ui| {
                        if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                            this.state = State::Finish;
                        }
                    })
                    .render(ui);
                    // ui.heading("Confirm Mnemonic");
                    // ui.label(" ");
                    // ui.label(" ");
                    // ui.separator();
                    // // ui.label(egui::RichText::new(mnemonic.phrase()).color(egui::Color32::WHITE));
                    // ui.separator();
                    // ui.label(" ");

                    // if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                    //     self.state = State::ConfirmMnemonic;
                    // }
                }

                State::Finish => {

                    panel(self)
                    .with_caption("Wallet Created")
                    .with_body(|_this,ui| {
                        ui.label(" ");
                        ui.label("Your wallet has been created and is ready to use.");
                        ui.label(" ");
                    })
                    .with_footer(move |this,ui| {
                        if ui.add_sized(size, egui::Button::new("Continue")).clicked() {
                            this.state = State::Start;
                            wallet.select::<section::Overview>();
                        }
                    })
                    .render(ui);

                    // ui.heading("Wallet Created");

                }

            }

        });
    }
}
