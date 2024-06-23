use crate::imports::*;
use super::*;

pub struct WalletSecret<'context> {
    context : &'context mut ManagerContext,
}

impl<'context> WalletSecret<'context> {

    pub fn new(context : &'context mut ManagerContext) -> Self {
        Self { context }
    }

    pub fn render(&mut self, ui : &mut Ui, core: &mut Core, rc : &RenderContext) -> bool {
        use egui_phosphor::light::{CHECK, X};

        let RenderContext { account, .. } = rc;

        let requires_payment_passphrase = account.requires_bip39_passphrase(core);
        let mut proceed_with_send = false;

        let response = TextEditor::new(
            &mut self.context.wallet_secret,
            &mut self.context.focus,
            Focus::WalletSecret,
            |ui, text| {
                ui.add_space(8.);
                ui.label(RichText::new("Enter wallet password").size(12.).raised());
                ui.add_sized(Overview::editor_size(ui), TextEdit::singleline(text)
                    .password(true)
                    .vertical_align(Align::Center))
            },
        )
        .build(ui);

        if response.text_edit_submit(ui) {
            if requires_payment_passphrase {
                self.context.focus.next(Focus::PaymentSecret);
            } else if !self.context.wallet_secret.is_empty() {
                proceed_with_send = true;
            }
        }

        if requires_payment_passphrase {
            let response = TextEditor::new(
                &mut self.context.payment_secret,
                &mut self.context.focus,
                Focus::PaymentSecret,
                |ui, text| {
                    ui.add_space(8.);
                    ui.label(RichText::new("Enter bip39 passphrase").size(12.).raised());
                    ui.add_sized(Overview::editor_size(ui), TextEdit::singleline(text)
                        .password(true)
                        .vertical_align(Align::Center))
                },
            )
            .build(ui);

            if response.text_edit_submit(ui) && !self.context.wallet_secret.is_empty() && !self.context.payment_secret.is_empty() {
                proceed_with_send = true;
            }
    
        }

        let is_ready_to_send = !(self.context.wallet_secret.is_empty() || requires_payment_passphrase && self.context.payment_secret.is_empty());

        ui.add_space(8.);
        CenterLayoutBuilder::new()
            .add_enabled(is_ready_to_send, Button::new(format!("{CHECK} Submit")).min_size(theme_style().medium_button_size()), |_this: &mut WalletSecret<'_>| {
                proceed_with_send = true;
            })
            .add(Button::new(format!("{X} Cancel")).min_size(theme_style().medium_button_size()), |this| {
                this.context.action = Action::Estimating;
                this.context.focus.next(Focus::Amount);
            })
            .build(ui,self);



        proceed_with_send
        
    }

}
