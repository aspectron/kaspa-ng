use crate::imports::*;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Select,
    Unlock { wallet_descriptor : WalletDescriptor, error : Option<Arc<Error>>},
    Unlocking { wallet_descriptor : WalletDescriptor },
}

pub struct WalletOpen {
    #[allow(dead_code)]
    runtime: Runtime,
    wallet_secret: String,
    pub state: State,
    pub message: Option<String>,
}

impl WalletOpen {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            wallet_secret: String::new(),
            state: State::Select,
            message: None,
        }
    }

    pub fn open(&mut self, wallet_descriptor: WalletDescriptor) {
        self.state = State::Unlock { wallet_descriptor, error : None};
    }

}

impl ModuleT for WalletOpen {

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Mobile
    }

    fn secure(&self) -> bool {
        true
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        let unlock_result = Payload::<Result<()>>::new("wallet_unlock_result");

        // let text: &str = i18n("Select a wallet to unlock");

        match self.state.clone() {
            State::Select => {

                let has_stack = core.has_stack();
                let core = Rc::new(RefCell::new(core));
                let mut wallet_descriptor_list = core.borrow_mut().wallet_list.clone();
                let wallet_descriptor_list_is_empty = wallet_descriptor_list.is_empty();

                Panel::new(self)
                    .with_caption(i18n("Select Wallet"))
                    .with_back_enabled(has_stack, |_| { core.borrow_mut().back() })
                    .with_header(|_ctx, ui| {
                        if !wallet_descriptor_list_is_empty {
                            ui.label(i18n("Select a wallet to unlock"));
                        } else {
                            ui.label(i18n("No wallets found, please create a new wallet"));
                        }
                    })
                    .with_body(|this, ui| {

                        if !wallet_descriptor_list_is_empty {
                            wallet_descriptor_list.sort();
                            for wallet_descriptor in wallet_descriptor_list.into_iter() {
                                if ui.add_sized(theme_style().large_button_size(), CompositeButton::image_and_text(
                                    Composite::icon(egui_phosphor::thin::FINGERPRINT_SIMPLE),
                                    wallet_descriptor.title.as_deref().unwrap_or_else(||i18n("NO NAME")),
                                    wallet_descriptor.filename.clone(),
                                )).clicked() {
                                    this.state = State::Unlock { wallet_descriptor : wallet_descriptor.clone(), error : None };
                                }
                            }
                            ui.label(" ");
                            ui.separator();
                        }

                        ui.label(" ");
                        if ui
                            .large_button(i18n("Create new wallet"))
                            .clicked()
                        {
                            core.borrow_mut().select::<modules::WalletCreate>();
                        }

                        ui.label(" ");
                    })
                    .render(ui);
            }

            State::Unlock{wallet_descriptor, error} => {

                let unlock = Rc::new(RefCell::new(false));

                Panel::new(self)
                    .with_caption(i18n("Unlock Wallet"))
                    .with_back(|ctx| {
                        ctx.state = State::Select;
                    })
                    .with_body(|ctx, ui| {
                        ui.label(format!(
                            "{} \"{}\"",
                            i18n("Opening wallet:"),
                            wallet_descriptor.title.as_deref().unwrap_or(wallet_descriptor.filename.as_str())
                        ));
                        ui.label(" ");

                        if let Some(err) = error {
                            ui.label(
                                RichText::new(err.to_string())
                                    .color(egui::Color32::from_rgb(255, 120, 120)),
                            );
                            ui.label(" ");
                        }

                        ui.label(i18n("Enter the password to unlock your wallet"));
                        ui.label(" ");


                        let response = ui.add_sized(
                                theme_style().panel_editor_size,
                                TextEdit::singleline(&mut ctx.wallet_secret)
                                    .password(true)
                                    .vertical_align(Align::Center),
                            );

                        if response.text_edit_submit(ui) {
                            *unlock.borrow_mut() = true;
                        } else {
                            response.request_focus();
                        }

                    })
                    .with_footer(|_,ui|{
                        if ui.large_button(i18n("Unlock")).clicked() {
                            *unlock.borrow_mut() = true;
                        }

                    })
                    .render(ui);

                    if *unlock.borrow() {
                        let wallet_secret = Secret::new(
                            self.wallet_secret.as_bytes().to_vec(),
                        );
                        self.wallet_secret.zeroize();
                        let wallet = self.runtime.wallet().clone();
                        let wallet_descriptor_delegate = wallet_descriptor.clone();
                        self.state = State::Unlocking { wallet_descriptor };
                        spawn_with_result(&unlock_result, async move {
                            sleep(Duration::from_secs(1)).await;
                            wallet.wallet_open(wallet_secret, Some(wallet_descriptor_delegate.filename), true, true).await?;
                            Ok(())
                        });
                    }

            }
            State::Unlocking { wallet_descriptor } => {
                ui.vertical_centered(|ui| {
                    ui.heading(i18n("Unlocking"));
                    ui.label(" ");
                    ui.label(i18n("Decrypting wallet, please wait..."));
                    ui.label(" ");
                    ui.add_space(64.);
                    ui.add(egui::Spinner::new().size(92.));

                    if let Some(result) = unlock_result.take() {
                        match result {
                            Ok(_) => {
                                // relocated to Core WalletUpdate event handler
                                // core.select::<modules::AccountManager>();
                                // self.state = Default::default();
                            }
                            Err(err) => {
                                // println!("Unlock error: {}", err);
                                self.state = State::Unlock { wallet_descriptor, error : Some(Arc::new(err)) };
                            }
                        }
                    }
                });
            }
        }
    }
}

fn _render_wallet_descriptor(wallet: &WalletDescriptor, ui: &mut Ui) -> LayoutJob {
    let mut job = LayoutJob {
        halign: Align::Center,
        ..Default::default()
    };

    job.append(
        wallet
            .title
            .clone()
            .unwrap_or_else(|| "NO NAME".to_string())
            .as_str(),
        0.0,
        TextFormat {
            font_id: FontId::new(18.0, FontFamily::Proportional),
            color: ui.ctx().style().visuals.strong_text_color(), //text_color(),
            ..Default::default()
        },
    );
    job.append(
        "\n",
        0.0,
        TextFormat {
            font_id: FontId::new(12.0, FontFamily::Proportional),
            color: ui.ctx().style().visuals.text_color(),
            ..Default::default()
        },
    );
    job.append(
        wallet.filename.clone().as_str(),
        0.0,
        TextFormat {
            font_id: FontId::new(12.0, FontFamily::Proportional),
            color: ui.ctx().style().visuals.text_color(),
            ..Default::default()
        },
    );

    job
}
