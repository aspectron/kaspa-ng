
#[allow(unused_imports)]
use kaspa_wallet_core::api::PrvKeyDataGetRequest;

use crate::imports::*;

#[derive(Clone)]
pub enum State {
    Select,
    SelectPrvKey,
    Export { error : Option<Arc<Error>> },
    Exporting,
    Mnemonic,
    Transportable,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
enum Focus {
    #[default]
    None,
    WalletSecret,
    PaymentSecret,
}

#[derive(Clone, Copy, Describe)]
pub enum ExportKind {
    // #[default]
    // None,
    Transportable,
    Mnemonic,
}

impl ExportKind {
    pub fn info(&self) -> (&'static str,&'static str) {
        match self {
            Self::Mnemonic => ("Private Key Mnemonic","Private key mnemonic stored in this wallet"),
            Self::Transportable => ("Transportable", "Encrypted hex encoded data easily importable into another instance of KaspaNG"),
        }
    }
}

pub enum ExportResult {
    Transportable(Arc<Vec<u8>>),
    Mnemonic(String),
}


#[derive(Default)]
struct Context {
    prv_key_data_id : Option<PrvKeyDataId>,
    // account_kind: Option<CreateAccountKind>,
    // _create_private_key: bool,
    // account_name: String,
    // enable_payment_secret: bool,
    wallet_secret : String,
    payment_secret: String,
    // payment_secret_confirm: String,
    focus : FocusManager<Focus>,
    kind : Option<ExportKind>,
}



pub struct Export {
    #[allow(dead_code)]
    runtime: Runtime,
    // wallet_secret: String,
    // payment_secret: String,
    pub state: State,
    pub message: Option<String>,
    context : Context,
    // kind: Option<ExportKind>,
    // prv_key_data_id : Option<PrvKeyDataId>,
    // focus : Focus,
}

impl Export {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            // wallet_secret: String::new(),
            // payment_secret: String::new(),
            state: State::Select,
            message: None,
            // kind: None,
            // prv_key_data_id: None,
            // focus : Focus::None,
            context : Default::default(),
            // selected_wallet: None,
        }
    }
}

impl ModuleT for Export {
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
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            // let size = egui::Vec2::new(200_f32, 40_f32);
            let export_result = Payload::<Result<()>>::new("wallet_export_result");

            match self.state.clone() {

                State::Select => {
                    let prv_key_data_list = core.prv_key_data_map.values().cloned().collect::<Vec<_>>();

                    let mut submit = false;
                    Panel::new(self)
                        .with_caption("Export")
                        .with_back_enabled(core.has_stack(),|_this| {
                            // this.state = State::KeySelection;
                            core.back();
                        })
                        // .with_close_enabled(false, |_|{
                        // })
                        .with_header(|_ctx,ui| {
                            ui.add_space(64.);
                            ui.label("Please select the type of export");
                            // ui.label("");
                        })
                        .with_body(|this,ui| {
                            // ui.label("(You can import additional private keys later, once the wallet has been created)");

                            for kind in ExportKind::list() {
                                let (name,info) = kind.info();
                                ui.horizontal(|ui| {
                                    if ui.large_button(name).clicked() {
                                        this.context.kind = Some(kind);
                                        submit = true;
                                    }
                                    ui.add_space(8.);
                                    ui.label(info);
                                    ui.add_space(16.);
                                });
                            }


                            // // ui.vertical(|ui| {
                            //     if ui.large_button("Private Key Mnemonic").clicked() {
                            //         this.context.kind = Some(ExportKind::Mnemonic);
                            //         submit = true;
                            //     }
                            //     ui.add_space(8.);
                            //     ui.label("Private key mnemonic stored in this wallet");
                            //     ui.add_space(16.);

                            //     if ui.large_button("Transportable").clicked() {
                            //         this.context.kind = Some(ExportKind::Transportable);
                            //         submit = true;
                            //     }
                            //     ui.add_space(8.);
                            //     ui.label("Portable encrypted hex-encoded data importable into another KaspaNG instance");
                            //     ui.add_space(16.);
                            // // });


                        })
                        .with_footer(|_this,_ui| {
                        })
                        .render(ui);

                        if submit {
                            match self.context.kind {
                                Some(ExportKind::Mnemonic) => {
                                    self.state = State::SelectPrvKey;
                                }
                                Some(ExportKind::Transportable) => {
                                    self.state = State::Export { error : None };
                                }
                                None => {
                                    panic!("Invalid export kind")
                                }
                            }
                            self.state = State::Export { error : None };
                            self.context.focus.next(Focus::WalletSecret);
                        }
                }

                State::SelectPrvKey => {
                    let prv_key_data_list = core.prv_key_data_map.values().cloned().collect::<Vec<_>>();

                    let mut submit = false;
                    Panel::new(self)
                        .with_caption("Export")
                        .with_back_enabled(core.has_stack(),|_this| {
                            // this.state = State::KeySelection;
                            core.back();
                        })
                        // .with_close_enabled(false, |_|{
                        // })
                        .with_header(|_ctx,ui| {
                            ui.add_space(64.);
                            ui.label("Please select the type of export");
                            // ui.label("");
                        })
                        .with_body(|this,ui| {
                            // ui.label("(You can import additional private keys later, once the wallet has been created)");

                            ui.label("");
                            ui.label("Please select private key to export");
                            ui.label("");

                            for prv_key_data_info in prv_key_data_list.into_iter() {
                                if ui.large_button(prv_key_data_info.name_or_id()).clicked() {
                                    this.context.prv_key_data_id = Some(*prv_key_data_info.id());
                                    submit = true;
                                }
                                ui.label("");
                            }

                        })
                        .with_footer(|_this,_ui| {
                        })
                        .render(ui);

                        if submit {
                            self.state = State::Export { error : None };
                            self.context.focus.next(Focus::WalletSecret);
                        }

                }

                State::Export{error} => {

                    // let mut submit_via_editor = false;
                    // let mut submit_via_footer = false;

                    Panel::new(self)
                        .with_caption("Unlock Wallet")
                        .with_back(|this| {
                            this.state = State::Select;
                        })
                        .with_body(|this, ui| {
                            // ui.label(format!(
                            //     "Opening wallet: \"{}\"",
                            //     wallet_descriptor.title.as_deref().unwrap_or(wallet_descriptor.filename.as_str())
                            // ));
                            // ui.label(" ");

                            if let Some(err) = error {
                                ui.label(
                                    egui::RichText::new(err.to_string())
                                        .color(egui::Color32::from_rgb(255, 120, 120)),
                                );
                                ui.label(" ");
                            }

                            ui.label(i18n("Enter the password for your wallet"));
                            ui.label(" ");

                            let mut execute = false;

                            // let response = ui.add_sized(
                            //         size,
                            //         TextEdit::singleline(&mut ctx.wallet_secret)
                            //             // .hint_text("Enter Password...")
                            //             .password(true)
                            //             .vertical_align(Align::Center),
                            //     );
                            // // ui.memory().request_focus(resp.id);
                            // if response.text_edit_submit(ui) {
                            //     unlock = true;
                            // } else {
                            //     response.request_focus();
                            // }

                            TextEditor::new(
                                &mut this.context.wallet_secret,
                                &mut this.context.focus,
                                Focus::WalletSecret,
                                |ui, text| {
                                    ui.label(egui::RichText::new("Enter your wallet secret").size(12.).raised());
                                    ui.add_sized(theme_style().panel_editor_size, TextEdit::singleline(text)
                                        .vertical_align(Align::Center)
                                        .password(true))
                                },
                            ).submit(|text,_focus| {
                                if !text.is_empty() {
                                    execute = true;
                                }
                            })
                            .build(ui);

                            if ui.large_button_enabled(this.context.wallet_secret.is_not_empty(),"Continue").clicked() {
                                execute = true;
                            }

                            if execute {



                                let wallet_secret = kaspa_wallet_core::secret::Secret::new(
                                    this.context.wallet_secret.as_bytes().to_vec(),
                                );
                                this.context.wallet_secret.zeroize();
                                let wallet = this.runtime.wallet().clone();
                                let prv_key_data_id = this.context.prv_key_data_id;
                                let export_kind = this.context.kind;
                                // let wallet_descriptor_delegate = wallet_descriptor.clone();
                                spawn_with_result(&export_result, async move {
                                    // wallet.wallet_open(wallet_secret, Some(wallet_descriptor_delegate.filename), true, true).await?;

                                    match export_kind {
                                        Some(ExportKind::Mnemonic) => {
                                            // wallet.export_mnemonic(prv_key_data_id.unwrap(), wallet_secret).await?;

                                            if let Some(prv_key_data_id) = prv_key_data_id {
                                                let result = wallet.prv_key_data_get(prv_key_data_id, wallet_secret).await?;
                                                // let prv_key_data = wallet.prv_key_data(prv_key_data_id).await?;
                                                // let mnemonic = prv_key_data.mnemonic().await?;
                                                // println!("mnemonic: {}", mnemonic);
                                            }

                                        }
                                        Some(ExportKind::Transportable) => {
                                            // wallet.export_transportable(prv_key_data_id.unwrap(), wallet_secret).await?;
                                        }
                                        None => {
                                            panic!("Invalid export kind")
                                        }
                                    }


                                    Ok(())
                                });

                                this.state = State::Exporting;
                            }

                            ui.label(" ");
                        })
                        .render(ui);
                }
                State::Exporting => {
                    ui.heading("Exporting");
                    ui.label(" ");
                    ui.label("Exporting... please wait...");
                    ui.label(" ");
                    ui.add_space(64.);
                    ui.add(egui::Spinner::new().size(92.));

                    if let Some(result) = export_result.take() {
                        match result {
                            Ok(_) => {
                                // println!("Export success");
                                // self.state = Default::default();
                                match self.context.kind {
                                    Some(ExportKind::Mnemonic) => {
                                        self.state = State::Mnemonic;
                                    }
                                    Some(ExportKind::Transportable) => {
                                        self.state = State::Transportable;
                                    }
                                    None => {
                                        panic!("Invalid export kind")
                                    }
                                }
                            }
                            Err(err) => {
                                println!("Unlock error: {}", err);
                                self.state = State::Export { error : Some(Arc::new(err)) };
                            }
                        }
                    }
                }

                State::Mnemonic => {
                    ui.label("mnemonic goes here");
                }
                
                State::Transportable => {
                    ui.label("transportable goes here");
                }

            }
        });
    }
}
