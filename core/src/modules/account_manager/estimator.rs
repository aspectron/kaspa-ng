use crate::imports::*;
use super::*;

pub struct Estimator<'context> {
    context: &'context mut ManagerContext
}

impl<'context> Estimator<'context> {
    pub fn new(context: &'context mut ManagerContext) -> Self {
        Self { context }
    }

    pub fn render(&mut self, core : &mut Core, ui: &mut Ui, rc : &RenderContext) -> bool {


        use egui_phosphor::light::{CHECK, X};

        let RenderContext { network_type, .. } = rc;

        let mut request_estimate = self.context.request_estimate.take().unwrap_or_default();

        match self.context.transaction_kind.as_ref().unwrap() {
            TransactionKind::Send => {
                Destination::new(self.context).render(core, ui, rc);
                // self.render_address_input(core, ui, rc);
            }
            TransactionKind::Transfer => {
                Transfer::new(self.context).render(core, ui, rc);
                // self.render_transfer_account_selector(core, ui, rc);
            }
        }

        let response = TextEditor::new(
            &mut self.context.send_amount_text,
            &mut self.context.focus,
            Focus::Amount,
            |ui, text| {
                ui.add_space(8.);
                ui.label(RichText::new(format!("{} {} {}", i18n("Enter"), kaspa_suffix(network_type), i18n("amount to send"))).size(12.).raised());
                ui.add_sized(Overview::editor_size(ui), TextEdit::singleline(text)
                    .vertical_align(Align::Center))
            },
        )
        .change(|_| {
            request_estimate = true;
        })
        .build(ui);

        if response.text_edit_submit(ui) {
            if self.context.enable_priority_fees {
                self.context.focus.next(Focus::Fees);
            } else if self.update_user_args() {
                self.context.action = Action::Sending;
                self.context.focus.next(Focus::WalletSecret);
            }
        }

        // TODO - improve the logic
        if core.network_pressure.is_high() {
            ui.label(format!("{}: {}% {} {} {:0.3} {} {}",
                i18n("The network is currently experiencing high load"),
                core.network_pressure.capacity(), 
                i18n("of its capacity."),
                i18n("It is recommended that you add a priority fee of at least"),
                0.001, 
                kaspa_suffix(network_type),
                i18n("to ensure faster transaction acceptance."),
            ));
        }

        ui.add_space(8.);
        if ui
            .checkbox(&mut self.context.enable_priority_fees,i18n("Include QoS Priority Fees"))
            // .on_hover_text_at_pointer(i18n("Add priority fees to ensure faster confirmation.\nUseful only if the network is congested."))
            .changed() {
            if self.context.enable_priority_fees {
                self.context.focus.next(Focus::Fees);
            } else {
                self.context.focus.next(Focus::Amount);
            }
        }

        if self.context.enable_priority_fees {
            TextEditor::new(
                &mut self.context.priority_fees_text,
                &mut self.context.focus,
                Focus::Fees,
                |ui, text| {
                    ui.add_space(8.);
                    ui.label(RichText::new("Enter priority fees").size(12.).raised());
                    ui.add_sized(Overview::editor_size(ui), TextEdit::singleline(text)
                        .vertical_align(Align::Center))
                },
            )
            .change(|_| {
                request_estimate = true;
            })
            .submit(|_,_|{
                self.context.action = Action::Sending;
            })
            .build(ui); 
        }

        ui.add_space(8.);
        let ready_to_send = match &*self.context.estimate.lock().unwrap() {
            EstimatorStatus::GeneratorSummary(estimate) => {
                if let Some(final_transaction_amount) = estimate.final_transaction_amount {
                    ui.label(format!("{} {}",i18n("Final Amount:"), sompi_to_kaspa_string_with_suffix(final_transaction_amount + estimate.aggregated_fees, network_type)));
                }
                let fee_title = if self.context.priority_fees_sompi != 0 {
                    i18n("Network and Priority Fees:")
                } else {
                    i18n("Network Fees:")
                };
                ui.label(format!("{} {}", fee_title, sompi_to_kaspa_string_with_suffix(estimate.aggregated_fees, network_type)));
                ui.label(format!("{} {} {} {}",i18n("Transactions:"), estimate.number_of_generated_transactions, i18n("UTXOs:"), estimate.aggregated_utxos));
                
                self.context.address_status == AddressStatus::Valid || (self.context.transaction_kind == Some(TransactionKind::Transfer) && self.context.transfer_to_account.is_some())
            }
            EstimatorStatus::Error(error) => {
                ui.label(RichText::new(error.to_string()).color(theme_color().error_color));
                false
            }
            EstimatorStatus::None => {
                ui.label(format!("{} {} {}", i18n("Please enter"), kaspa_suffix(network_type), i18n("amount to send")));
                false
            }
        };
        ui.add_space(8.);

        ui.horizontal(|ui| {
            ui.vertical_centered(|ui|{
                ui.horizontal(|ui| {
                    CenterLayoutBuilder::new()
                        .add_enabled(ready_to_send, Button::new(format!("{CHECK} {}", i18n("Send"))).min_size(theme_style().medium_button_size()), |this: &mut Estimator<'_>| {
                            this.context.action = Action::Sending;
                            this.context.focus.next(Focus::WalletSecret);
                        })
                        .add(Button::new(format!("{X} {}", i18n("Cancel"))).min_size(theme_style().medium_button_size()), |this| {
                            this.context.reset_send_state();
                        })
                        .build(ui, self)
                });
            });

        });

        self.update_user_args() 
            && request_estimate 
            && matches!(self.context.action,Action::Estimating)

    }



    fn update_user_args(&mut self) -> bool {
        let mut valid = true;

        match try_kaspa_str_to_sompi(self.context.send_amount_text.as_str()) {
            Ok(Some(sompi)) => {
                self.context.send_amount_sompi = sompi;
            }
            Ok(None) => {
                self.user_error(i18n("Please enter an amount").to_string());
                valid = false;
            }
            Err(err) => {
                self.user_error(format!("{} {err}", i18n("Invalid amount:")));
                valid = false;
            }
        }

        match try_kaspa_str_to_sompi(self.context.priority_fees_text.as_str()) {
            Ok(Some(sompi)) => {
                self.context.priority_fees_sompi = sompi;
            }
            Ok(None) => {
                self.context.priority_fees_sompi = 0;
            }
            Err(err) => {
                self.user_error(format!("{} {err}", i18n("Invalid fee amount:")));
                valid = false;
            }
        }

        valid
    }

    fn user_error(&self, error : impl Into<String>) {
        *self.context.estimate.lock().unwrap() = EstimatorStatus::Error(error.into());
    }
        
}