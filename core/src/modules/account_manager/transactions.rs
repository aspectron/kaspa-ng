use crate::imports::*;
use super::*;
use crate::core::TRANSACTION_PAGE_SIZE;

pub struct Transactions { }

impl Transactions {
    pub fn new() -> Self {
        Self { }
    }

    pub fn render(&mut self, ui: &mut Ui, core : &mut Core, rc : &RenderContext) {
        let RenderContext { account, network_type, current_daa_score, .. } = rc;

        egui::ScrollArea::vertical().auto_shrink([false,false]).show(ui, |ui| {
            let transactions = account.transactions();
            if transactions.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.label("");
                    ui.label(RichText::new(i18n("No transactions")).size(16.));
                });
            } else {
                let total: u64 = transactions.iter().map(|transaction|transaction.aggregate_input_value()).sum();
                transactions.iter().for_each(|transaction| {
                    transaction.render(ui, *network_type, *current_daa_score, true, Some(total));
                });
            }
        });

        ui.add_space(4.);
        let pagination = Pagination::new(account.transaction_count(), account.transaction_start().into(), Some(TRANSACTION_PAGE_SIZE), Some(5));
        if let Some(start) = pagination.render(ui){
            core.load_account_transactions_with_range(account, start..(start+TRANSACTION_PAGE_SIZE))
                    .map_err(|err|{
                        log_info!("Failed to load transactions\n{err:?}")
                    }).ok();

            account.set_transaction_start(start);
            runtime().request_repaint();
        }
        // ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        //     let mut start = None;
        //     let total = account.transaction_count();
        //     //log_info!("DEBUG: Transaction Count: {total:?}");
        //     let list_start = account.transaction_start();
        //     if ui.medium_button_enabled(list_start>0, i18n("Prev")).clicked() {
        //         start = Some(list_start - TRANSACTION_PAGE_SIZE);
        //     }
        //     if ui.medium_button_enabled(list_start + TRANSACTION_PAGE_SIZE < total, i18n("Next")).clicked() {
        //         start = Some(list_start + TRANSACTION_PAGE_SIZE);
        //     }
            

        //     if let Some(start) = start{
        //         core.load_account_transactions_with_range(account, start..(start+TRANSACTION_PAGE_SIZE))
        //                 .map_err(|err|Error::custom(format!("Failed to load transactions\n{err}")))?;

        //         account.set_transaction_start(start);
        //         runtime().request_repaint();    
        //     }

        //     Ok::<(), Error>(())
        // });
    }
}
