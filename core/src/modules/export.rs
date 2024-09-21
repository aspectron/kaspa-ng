use crate::imports::*;

#[derive(Clone)]
pub enum State {
    Select,
    SelectPrvKey,
    Authenticate,
    Export,
    Error { error : Arc<Error> },
    Exporting,
    Mnemonic { mnemonic : String },
    Transportable { data : Arc<Vec<u8>> },
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
enum Focus {
    #[default]
    None,
    WalletSecret,
    PaymentSecret,
}

#[derive(Default, Clone, Copy, Describe, Eq, PartialEq)]
pub enum ExportKind {
    #[default]
    Mnemonic,
    Transportable,
}

impl ExportKind {
    pub fn info(&self) -> (&'static str,&'static str) {
        match self {
            Self::Mnemonic => (i18n("Private Key Mnemonic"),i18n("Private key mnemonic stored in this wallet")),
            Self::Transportable => (i18n("Transportable"), i18n("Encrypted hex encoded data easily importable into another instance of Kaspa-NG")),
        }
    }
}

impl Zeroize for ExportKind {
    fn zeroize(&mut self) {
        *self = Self::Mnemonic;
    }
}

#[derive(Clone)]
pub enum ExportResult {
    Transportable(Arc<Vec<u8>>),
    Mnemonic(String),
}


#[derive(Default)]
struct Context {
    prv_key_data_info : Option<Arc<PrvKeyDataInfo>>,
    wallet_secret : String,
    payment_secret: String,
    mnemonic_presenter_context : MnemonicPresenterContext,
    kind : ExportKind,
    focus : FocusManager<Focus>,
}

impl Zeroize for Context {
    fn zeroize(&mut self) {
        self.wallet_secret.zeroize();
        self.payment_secret.zeroize();
        self.mnemonic_presenter_context.zeroize();
        self.kind.zeroize();
        self.focus.clear();
    }
}

pub struct Export {
    #[allow(dead_code)]
    runtime: Runtime,
    pub state: State,
    context : Context,
}

impl Export {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            state: State::Select,
            context : Default::default(),
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

            let export_result = Payload::<Result<ExportResult>>::new("wallet_export_result");

            match self.state.clone() {

                State::Select => {

                    let mut submit = false;
                    Panel::new(self)
                        .with_caption("Export")
                        .with_back_enabled(core.has_stack(),|_this| {
                            core.back();
                        })
                        .with_header(|_ctx,ui| {
                            ui.add_space(64.);
                            ui.label(i18n("Please select export type"));
                        })
                        .with_body(|this,ui| {

                            ui.vertical_centered(|ui| {
                                for kind in ExportKind::into_iter() {
                                    if kind == ExportKind::Transportable {
                                        continue;
                                    }
                                    let (_name,info) = kind.info();
                                    ui.radio_value(&mut this.context.kind, kind, info);
                                }
                            });

                        })
                        .with_footer(|_this,ui| {
                            if ui.large_button(i18n("Continue")).clicked() {
                                submit = true;
                            }
                        })
                        .render(ui);

                        if submit {
                            match self.context.kind {
                                ExportKind::Mnemonic => {
                                    self.state = State::SelectPrvKey;
                                }
                                ExportKind::Transportable => {
                                    // self.state = State::Transportable;
                                    // self.state = State::Export { error : None };
                                }
                            }
                        }
                }

                State::SelectPrvKey => {
                    let prv_key_data_map = core.prv_key_data_map.clone();

                    let mut submit = false;
                    Panel::new(self)
                        .with_caption("Export")
                        .with_back_enabled(core.has_stack(),|_this| {
                            core.back();
                        })
                        .with_header(|_ctx,ui| {
                            ui.add_space(64.);
                            ui.label(i18n("Please select the private key to export"));
                        })
                        .with_body(|this,ui| {
                            if let Some(prv_key_data_map) = prv_key_data_map {
                                for prv_key_data_info in prv_key_data_map.values() {
                                    if ui.large_button(prv_key_data_info.name_or_id()).clicked() {
                                        this.context.prv_key_data_info = Some(prv_key_data_info.clone());
                                        submit = true;
                                    }
                                    ui.label("");
                                }
                            }

                        })
                        .with_footer(|_this,_ui| {
                        })
                        .render(ui);

                        if submit {
                            self.state = State::Authenticate;
                            self.context.focus.next(Focus::WalletSecret);
                        }
                }

                State::Authenticate => {
                    let submit = Rc::new(RefCell::new(false));

                    let requires_bip39_passphrase = self.context.prv_key_data_info.as_ref().unwrap().requires_bip39_passphrase();

                    Panel::new(self)
                        .with_caption(i18n("Unlock Wallet"))
                        .with_back(|this| {
                            this.state = State::Select;
                        })
                        .with_body(|this, ui| {
                            ui.label(i18n("Enter the password for your wallet"));
                            ui.label(" ");

                            TextEditor::new(
                                &mut this.context.wallet_secret,
                                &mut this.context.focus,
                                Focus::WalletSecret,
                                |ui, text| {
                                    ui.label(RichText::new("Enter your wallet secret").size(12.).raised());
                                    ui.add_sized(theme_style().panel_editor_size, TextEdit::singleline(text)
                                        .vertical_align(Align::Center)
                                        .password(true))
                                },
                            ).submit(|text,focus| {
                                if !text.is_empty() && requires_bip39_passphrase {
                                    focus.next(Focus::PaymentSecret);
                                } else if !text.is_empty() {
                                    *submit.borrow_mut() = true;
                                }
                            })
                            .build(ui);

                            ui.label(" ");

                            if requires_bip39_passphrase {

                                TextEditor::new(
                                    &mut this.context.payment_secret,
                                    &mut this.context.focus,
                                    Focus::PaymentSecret,
                                    |ui, text| {
                                        ui.label(RichText::new("Enter your payment secret").size(12.).raised());
                                        ui.add_sized(theme_style().panel_editor_size, TextEdit::singleline(text)
                                            .vertical_align(Align::Center)
                                            .password(true))
                                    },
                                ).submit(|text,focus| {
                                    if !text.is_empty() {
                                        focus.clear();
                                    }
                                })
                                .build(ui);

                            }

                        })
                        .with_footer(|this,ui| {
                            let ok = this.context.wallet_secret.is_not_empty() && (!requires_bip39_passphrase || this.context.payment_secret.is_not_empty());
                            if ui.large_button_enabled(ok, i18n("Continue")).clicked() {
                                *submit.borrow_mut() = true;
                            }
                        })
                        .render(ui);

                        if *submit.borrow() {
                            self.state = State::Export;
                            self.context.focus.clear();
                        }
                }

                State::Export => {


                    let wallet_secret = Secret::new(self.context.wallet_secret.as_str().into());
                    let requires_bip39_passphrase = self.context.prv_key_data_info.as_ref().unwrap().requires_bip39_passphrase();
                    let payment_secret: Option<Secret> = requires_bip39_passphrase
                        .then(|| self.context.payment_secret.as_str().into());
                    self.context.wallet_secret.zeroize();
                    let wallet = self.runtime.wallet().clone();
                    let prv_key_data_info = self.context.prv_key_data_info.clone();
                    let export_kind = self.context.kind;
                    spawn_with_result(&export_result, async move {

                        match export_kind {
                            ExportKind::Mnemonic => {

                                if let Some(prv_key_data_info) = prv_key_data_info {
                                    let prv_key_data = wallet.prv_key_data_get(*prv_key_data_info.id(), wallet_secret).await?;
                                    let mnemonic = prv_key_data.as_mnemonic(payment_secret.as_ref())?.ok_or(Error::custom("No mnemonic available"))?;
                                    Ok(ExportResult::Mnemonic(mnemonic.phrase_string()))
                                } else {
                                    Err(Error::custom("No private key data available"))
                                }

                            }
                            ExportKind::Transportable => {
                                Ok(ExportResult::Transportable(Arc::new(vec![])))
                            }
                        }
                    });

                    self.state = State::Exporting;
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
                            Ok(kind) => {
                                match kind {
                                    ExportResult::Mnemonic(mnemonic) => {
                                        self.state = State::Mnemonic { mnemonic };
                                    }
                                    ExportResult::Transportable(data) => {
                                        self.state = State::Transportable { data };
                                    }
                                }
                            }
                            Err(err) => {
                                self.state = State::Error { error : Arc::new(err) };
                            }
                        }
                    }
                }

                State::Error { error } => {
                    self.context.zeroize();

                    Panel::new(self)
                        .with_caption("Export Error")
                        .with_back(|this| {
                            this.state = State::Select;
                        })
                        .with_body(|_this, ui| {
                            ui.label(
                                RichText::new(error.to_string())
                                    .color(egui::Color32::from_rgb(255, 120, 120)),
                            );
                            ui.label(" ");
                        })
                        .with_footer(|this,ui| {
                            if ui.large_button("Restart").clicked() {
                                this.state = State::Select;
                            }
                        })
                        .render(ui);
                }

                State::Mnemonic { mnemonic } => {
                    Panel::new(self)
                        .with_caption("Mnemonic Export")
                        .with_body(|this, ui| {
                            
                            let mut mnemonic_presenter = MnemonicPresenter::new(mnemonic.as_str(), &mut this.context.mnemonic_presenter_context);
                            ui.label("");
                            mnemonic_presenter.render(ui, Some(i18n("Your private key mnemonic is:")));
                            ui.label("");
                                    
                        })
                        .with_footer(|this,ui| {
                            if ui.large_button(i18n("Continue")).clicked() {
                                this.context.zeroize();
                                this.state = State::Select;
                                core.select::<modules::AccountManager>();
                            }
                        })
                        .render(ui);
                }
                
                State::Transportable { data : _ } => {
                    ui.label("Transportable export is not yet implemented");
                }

            }
        });
    }
}
