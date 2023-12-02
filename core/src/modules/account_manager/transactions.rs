use crate::imports::*;
use super::*;

pub struct Transactions { }

impl Transactions {
    pub fn new() -> Self {
        Self { }
    }

    pub fn render(&mut self, ui: &mut Ui, _core : &mut Core, rc : &RenderContext<'_>) {
        let RenderContext { account, network_type, current_daa_score, .. } = rc;

        egui::ScrollArea::vertical().auto_shrink([false,false]).show(ui, |ui| {
            let transactions = account.transactions();
            if transactions.is_empty() {
                ui.label("No transactions");
            } else {
                let total: u64 = transactions.iter().map(|transaction|transaction.aggregate_input_value()).sum();
                transactions.iter().for_each(|transaction| {
                    transaction.render(ui, *network_type, *current_daa_score, true, Some(total));
                });
            }
        });
    }
}
