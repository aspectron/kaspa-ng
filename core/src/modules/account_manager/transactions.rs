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
        let max_height = ui.available_height() - (ui.fonts(|fonts|RichText::new("YWgy").font_height(fonts, ui.style())).at_least(ui.spacing().interact_size.y) * 2.0 + 5.0);
        egui::ScrollArea::vertical().max_height(max_height).auto_shrink([false,false]).show(ui, |ui| {
            let transactions = account.transactions();
            if transactions.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.label("");
                    ui.label(RichText::new(i18n("No transactions")).size(16.));
                });
            } else {
                let total: u64 = transactions.iter().map(|transaction|transaction.aggregate_input_value()).sum();
                transactions.iter().for_each(|transaction| {
                    transaction.render(ui, *network_type, account.network(), *current_daa_score, true, Some(total));
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
    }
}
