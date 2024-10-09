use egui_phosphor::thin::SEAL_WARNING;

use crate::imports::*;
use crate::utils::{secret_score, render_secret_score_text};

#[derive(Clone)]
pub enum State {
    Start,
    WalletSecret,
    Processing,
    Error { error : Arc<Error> },
    Finish,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
enum Focus {
    #[default]
    None,
    OldWalletSecret,
    NewWalletSecret,
    NewWalletSecretConfirm,
}

#[derive(Default)]
struct WalletSecretContext {
    old_wallet_secret: String,
    new_wallet_secret: String,
    new_wallet_secret_confirm: String,
    show_secrets: bool,
    new_wallet_secret_score: Option<f64>,
}

impl Zeroize for WalletSecretContext {
    fn zeroize(&mut self) {
        self.old_wallet_secret.zeroize();
        self.new_wallet_secret.zeroize();
        self.new_wallet_secret_confirm.zeroize();
        self.show_secrets = false;
        self.new_wallet_secret_score = None;
    }
}

pub struct WalletSecret {
    #[allow(dead_code)]
    runtime: Runtime,
    context: WalletSecretContext,
    state: State,
    focus: FocusManager<Focus>,
}

impl Zeroize for WalletSecret {
    fn zeroize(&mut self) {
        self.context.zeroize();
        self.state = State::Start;
        self.focus.next(Focus::None);
    }
}

impl WalletSecret {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            context: WalletSecretContext::default(),
            state: State::Start,
            focus: FocusManager::default(),
        }
    }
}

impl ModuleT for WalletSecret {

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Mobile
    }

    fn modal(&self) -> bool {
        true
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        let secret_change_result = Payload::<Result<()>>::new("secret_change_result");

        match self.state.clone() {
            State::Start => {
                let back = Rc::new(RefCell::new(false));
                if !core.state().is_open() {
                    Panel::new(self)
                    .with_caption(i18n("Change Wallet Secret"))
                        .with_back(|_this| {
                            *back.borrow_mut() = true;
                        })
                        .with_header(|_ctx,_ui| {
                        })
                        .with_body(|_this,ui| {
                            ui.add_space(16.);                                    
                            ui.label(
                                RichText::new(SEAL_WARNING)
                                    .size(theme_style().icon_size_large)
                                    .color(theme_color().error_color)
                            );
                            ui.add_space(16.);                                    
                            ui.label(i18n("This feature requires an open wallet"));
                        })
                        .with_footer(|_,ui| {
                            if ui.large_button(i18n("Close")).clicked() {
                                *back.borrow_mut() = true;
                            }
                        })
                        .render(ui);

                    if *back.borrow() {
                        core.back();
                    }

                } else {
                    self.state = State::WalletSecret;
                }
            }
            State::WalletSecret => {


                let back = Rc::new(RefCell::new(false));
                let mut submit = false;
                let mut allow = true;
                // let mut back = false;

                Panel::new(self)
                    .with_caption(i18n("Change Wallet Secret"))
                    .with_back(|_this| {
                        *back.borrow_mut() = true;
                    })
                    .with_close_enabled(false, |_|{
                        // *back.borrow_mut() = true
                    })
                    .with_header(|_ctx,_ui| {
                    })
                    .with_body(|this,ui| {
                        TextEditor::new(
                            &mut this.context.old_wallet_secret,
                            &mut this.focus,
                            Focus::OldWalletSecret,
                            |ui, text| {
                                ui.label(RichText::new(i18n("Enter your current wallet secret")).size(12.).raised());
                                ui.add_sized(theme_style().panel_editor_size, TextEdit::singleline(text)
                                    .vertical_align(Align::Center)
                                    .password(!this.context.show_secrets))
                            },
                        ).submit(|text,focus| {
                            if !text.is_empty() {
                                focus.next(Focus::NewWalletSecret)
                            }
                        })
                        .build(ui);

                        ui.add_space(32.);

                        let mut change = false;

                        TextEditor::new(
                            &mut this.context.new_wallet_secret,
                            &mut this.focus,
                            Focus::NewWalletSecret,
                            |ui, text| {
                                ui.label(RichText::new(i18n("Enter new wallet secret")).size(12.).raised());
                                ui.add_sized(theme_style().panel_editor_size, TextEdit::singleline(text)
                                    .vertical_align(Align::Center)
                                    .password(!this.context.show_secrets))
                            },
                        )
                        .change(|_|{
                            change = true;
                        })
                        .submit(|text,focus| {
                            if !text.is_empty() {
                                focus.next(Focus::NewWalletSecretConfirm)
                            }
                        })
                        .build(ui);

                        ui.add_space(8.);
                        TextEditor::new(
                            &mut this.context.new_wallet_secret_confirm,
                            &mut this.focus,
                            Focus::NewWalletSecretConfirm,
                            |ui, text| {
                                ui.label(RichText::new(i18n("Validate new wallet secret")).size(12.).raised());
                                ui.add_sized(theme_style().panel_editor_size, TextEdit::singleline(text)
                                    .vertical_align(Align::Center)
                                    .password(!this.context.show_secrets))
                            },
                        )
                        .change(|_|{
                            change = true;
                        })
                        .submit(|text,focus| {
                            if !text.is_empty() {
                                focus.next(Focus::None)
                            }
                        })
                        .build(ui);

                        ui.add_space(24.);
                        
                        ui.checkbox(&mut this.context.show_secrets, i18n("Show secrets in clear text"));
                        ui.add_space(16.);

                        if change {
                            let wallet_secret = this
                                .context
                                .new_wallet_secret
                                .is_not_empty()
                                .then_some(this.context.new_wallet_secret.clone())
                                .or(this.context
                                    .new_wallet_secret_confirm
                                    .is_not_empty()
                                    .then_some(this.context.new_wallet_secret_confirm.clone())
                                );
                            this.context.new_wallet_secret_score = wallet_secret.map(secret_score); //Some(password_score(&this.context.wallet_secret));
                        }

                        if let Some(score) = this.context.new_wallet_secret_score {
                            ui.label("");
                            render_secret_score_text(ui, i18n("Secret score:"), score);
                            if score < 80.0 && !core.settings.developer.password_restrictions_disabled() {
                                allow = false;
                                ui.label(RichText::new(i18n("Please enter a stronger secret")).color(error_color()));
                            }
                            ui.label("");
                        } else if this.context.new_wallet_secret_confirm.is_not_empty() && this.context.new_wallet_secret != this.context.new_wallet_secret_confirm {
                            ui.label(" ");
                            ui.label(RichText::new(i18n("Secrets do not match")).color(error_color()));
                            ui.label(" ");
                            allow = false;
                        } else {
                            ui.label(" ");
                        }

                    })
                    .with_footer(|this,ui| {
                        let enabled = this.context.old_wallet_secret.is_not_empty() &&
                            this.context.new_wallet_secret.is_not_empty() &&
                            this.context.new_wallet_secret == this.context.new_wallet_secret_confirm;

                        if ui.large_button_enabled(enabled,i18n("Change Secret")).clicked() {
                            submit = true;
                        }
                    })
                    .render(ui);

                if *back.borrow() {
                    self.zeroize();
                    core.back();
                } else if submit {
                    self.state = State::Processing;
                    self.focus.next(Focus::None);
                }

            }
            State::Processing => {

                Panel::new(self)
                    .with_caption(i18n("Change Wallet Secret"))
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.label(i18n("Processing..."));
                    })
                    .with_body(|_this,ui| {


                        ui.add_space(64.);
                        ui.add(egui::Spinner::new().size(92.));

                    })
                    .render(ui);

                    if !secret_change_result.is_pending() {
                        let old_wallet_secret = Secret::from(self.context.old_wallet_secret.as_str());
                        let new_wallet_secret = Secret::from(self.context.new_wallet_secret.as_str());
                        let wallet = self.runtime.wallet().clone();
                        spawn_with_result(&secret_change_result, async move {
                            wallet.wallet_change_secret(old_wallet_secret, new_wallet_secret).await?;
                            Ok(())
                        });
                    }

                    if let Some(result) = secret_change_result.take() {
                        match result {
                            Ok(()) => {
                                self.state = State::Finish;
                                self.context.zeroize();
                            }
                            Err(err) => {
                                self.state = State::Error { error : Arc::new(err) };
                            }
                        }
                    }
            }

            State::Error { error } => {

                Panel::new(self)
                    .with_caption(i18n("Change Wallet Secret"))
                    .with_header(|_ctx,_ui| {
                    })
                    .with_body(|_this,ui| {

                        ui.label(
                            RichText::new(SEAL_WARNING)
                                .size(theme_style().icon_size_large)
                                .color(theme_color().error_color)
                        );
                        ui.add_space(8.);                                    
                        ui.colored_label(error_color(), format!("{error}"));

                    })
                    .with_footer(|this,ui| {
                        if ui.large_button(i18n("Retry")).clicked() {
                            this.state = State::WalletSecret;
                            this.focus.next(Focus::NewWalletSecret);
                        }
                        if ui.large_button(i18n("Close")).clicked() {
                            this.zeroize();
                            if core.has_stack() {
                                core.back();
                            } else {
                                core.select::<modules::AccountManager>();
                            }
                        }
                    })
                    .render(ui);
            }

            State::Finish => {

                Panel::new(self)
                    .with_caption(i18n("Change Wallet Secret"))
                    .with_header(|_ctx,_ui| {
                    })
                    .with_body(|_this,ui| {

                        ui.label(i18n("The wallet secret has been changed successfully."));

                    })
                    .with_footer(|this,ui| {
                        if ui.large_button(i18n("Close")).clicked() {
                            this.zeroize();
                            if core.has_stack() {
                                core.back();
                            } else {
                                core.select::<modules::AccountManager>();
                            }
                        }
                    })
                    .render(ui);
            }
        }
    }
}
