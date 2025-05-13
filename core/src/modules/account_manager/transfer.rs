use crate::imports::*;
use super::*;

pub struct Transfer<'context> {
    context : &'context mut ManagerContext,
}

impl<'context> Transfer<'context> {
    pub fn new(context : &'context mut ManagerContext) -> Self {
        Self { context }
    }

    pub fn render(&mut self, core: &mut Core, ui : &mut Ui, rc : &RenderContext) {

        let RenderContext { network_type, .. } = rc;

        let default_account = core.account_collection().as_ref().and_then(|collection|{
            if collection.len() <= 1 {
                unreachable!("expecting least 2 accounts");
            }
            if collection.len() == 2 {
                collection.list().iter().find(|account|account.id() != rc.account.id()).cloned()
            } else {
                None
            }
        });

        if let Some(account) = default_account {
            self.context.transfer_to_account = Some(account.clone());
            // ui.label(format!("Transferring funds to: {}", account.name_or_id()));
            ui.label(i18n_args("Transferring funds to: {account}", &[("account", account.name_or_id())]));
            // ui.label(format!("Destination balance: {}", sompi_to_kaspa_string_with_suffix(account.balance().map(|balance|balance.mature).unwrap_or(0), network_type)));
            ui.label(i18n_args("Destination balance: {balance}", &[("balance", sompi_to_kaspa_string_with_suffix(account.balance().map(|balance|balance.mature).unwrap_or(0), network_type))]));
        } else {

            if self.context.transfer_to_account.as_ref().map(|account|account.id() == rc.account.id()).unwrap_or_default() {
                self.context.transfer_to_account = None;
                self.context.transfer_to_account.take();
            }

            let transfer_to_account = self.context.transfer_to_account.clone();

            PopupPanel::new(PopupPanel::id(ui,"transfer_selector_popup"),|ui|{ 
                let response = ui.vertical_centered(|ui| {
                    if let Some(account) = transfer_to_account {
                        // let response = ui.add(Label::new(format!("Transferring funds to: {} ⏷", account.name_or_id())).sense(Sense::click()));
                        let response = ui.add(Label::new(i18n_args("Transferring funds to: {account}", &[("account", account.name_or_id())])).sense(Sense::click()));
                        // ui.label(format!("Destination balance: {}", sompi_to_kaspa_string_with_suffix(account.balance().map(|balance|balance.mature).unwrap_or(0), network_type)));
                        ui.label(i18n_args("Destination balance: {balance}", &[("balance", sompi_to_kaspa_string_with_suffix(account.balance().map(|balance|balance.mature).unwrap_or(0), network_type))]));
                        response
                    } else if self.context.send_amount_text.is_not_empty() {
                        ui.add(Label::new(RichText::new(i18n("Please select destination account ⏷")).color(theme_color().warning_color)).sense(Sense::click()))
                    } else {
                        ui.add(Label::new(RichText::new(i18n("Please select destination account ⏷"))).sense(Sense::click()))
                    }
                });

                response.inner
            }, |ui, _| {

                egui::ScrollArea::vertical()
                    .id_salt("transfer_selector_popup_scroll")
                    .auto_shrink([true; 2])
                    .show(ui, |ui| {

                        if let Some(account_collection) = core.account_collection() {
                            account_collection.iter().for_each(|account| {
                                if account.id() == rc.account.id() {
                                    return;
                                }

                                if ui.account_selector_button(account, network_type, false, core.balance_padding()).clicked() {
                                    self.context.transfer_to_account = Some(account.clone());
                                }
                            });
                        }

                    });

            })
            .with_min_width(240.)
            .with_close_on_interaction(true)
            .build(ui);
        }

    }
}