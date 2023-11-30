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
    // selected_wallet: Option<String>,
}

impl WalletOpen {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            wallet_secret: String::new(),
            state: State::Select,
            message: None,
            // selected_wallet: None,
        }
    }

    pub fn open(&mut self, wallet_descriptor: WalletDescriptor) {
        self.state = State::Unlock { wallet_descriptor, error : None};
    }

    // pub fn lock(&mut self) {
    //     // Go to unlock page
    //     self.state = State::Unlock(None);
    // }
}

impl ModuleT for WalletOpen {

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

        let size = egui::Vec2::new(200_f32, 40_f32);
        let unlock_result = Payload::<Result<()>>::new("wallet_unlock_result");

        let text: &str = "Select a wallet to unlock";

        match self.state.clone() {
            State::Select => {
                Panel::new(self)
                    .with_caption("Select Wallet")
                    .with_close_enabled(false, |_| {})
                    .with_header(|_ctx, ui| {
                        ui.label(text);
                    })
                    .with_body(|this, ui| {
                        for wallet_descriptor in core.wallet_list.iter() {
                            if ui.add_sized(size, CompositeButton::new(
                                wallet_descriptor.title.as_deref().unwrap_or("NO NAME"),
                                wallet_descriptor.filename.clone(),
                            )).clicked() {
                                this.state = State::Unlock { wallet_descriptor : wallet_descriptor.clone(), error : None };
                            }
                        }
                        ui.label(" ");
                        ui.separator();
                        ui.label(" ");
                        if ui
                            .large_button("Create new wallet")
                            .clicked()
                        {
                            core.select::<modules::WalletCreate>();
                        }

                        ui.label(" ");
                    })
                    .render(ui);
            }

            State::Unlock{wallet_descriptor, error} => {
                Panel::new(self)
                    .with_caption("Unlock Wallet")
                    .with_back(|ctx| {
                        ctx.state = State::Select;
                    })
                    .with_body(|ctx, ui| {
                        ui.label(format!(
                            "Opening wallet: \"{}\"",
                            wallet_descriptor.title.as_deref().unwrap_or(wallet_descriptor.filename.as_str())
                        ));
                        ui.label(" ");

                        if let Some(err) = error {
                            ui.label(
                                egui::RichText::new(err.to_string())
                                    .color(egui::Color32::from_rgb(255, 120, 120)),
                            );
                            ui.label(" ");
                        }

                        ui.label(i18n("Enter the password to unlock your wallet"));
                        ui.label(" ");

                        let mut unlock = false;

                        let response = ui.add_sized(
                                size,
                                TextEdit::singleline(&mut ctx.wallet_secret)
                                    .password(true)
                                    .vertical_align(Align::Center),
                            );
                        // ui.memory().request_focus(resp.id);
                        if response.text_edit_submit(ui) {
                            unlock = true;
                        } else {
                            response.request_focus();
                        }

                        if ui.add_sized(size, egui::Button::new("Unlock")).clicked() {
                            unlock = true;
                        }

                        if unlock {
                            let wallet_secret = kaspa_wallet_core::secret::Secret::new(
                                ctx.wallet_secret.as_bytes().to_vec(),
                            );
                            ctx.wallet_secret.zeroize();
                            let wallet = ctx.runtime.wallet().clone();
                            let wallet_descriptor_delegate = wallet_descriptor.clone();
                            spawn_with_result(&unlock_result, async move {
                                wallet.wallet_open(wallet_secret, Some(wallet_descriptor_delegate.filename), true, true).await?;
                                Ok(())
                            });

                            ctx.state = State::Unlocking { wallet_descriptor };
                        }

                        ui.label(" ");
                    })
                    .render(ui);
            }
            State::Unlocking { wallet_descriptor } => {
                ui.heading("Unlocking");
                ui.label(" ");
                ui.label("Unlocking wallet, please wait...");
                ui.label(" ");
                ui.add_space(64.);
                ui.add(egui::Spinner::new().size(92.));

                if let Some(result) = unlock_result.take() {
                    match result {
                        Ok(_) => {
                            println!("Unlock success");
                            core.select::<modules::AccountManager>();
                            self.state = Default::default();
                        }
                        Err(err) => {
                            println!("Unlock error: {}", err);
                            self.state = State::Unlock { wallet_descriptor, error : Some(Arc::new(err)) };
                        }
                    }
                }
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
