use crate::imports::*;
use super::*;

pub struct BalancePane<'context> {
    context : &'context ManagerContext,
}

impl<'context> BalancePane<'context> {
    
    pub fn new(context : &'context ManagerContext) -> Self {
        Self { context }
    }
    
    pub fn render(&mut self, core: &mut Core, ui : &mut Ui, rc : &RenderContext<'_>) {
    

        // let theme = theme();
        let RenderContext { account, network_type, .. } = rc;

        ui.add_space(10.);

        if let Some(balance) = account.balance() {
            
            if !core.state().is_synced() {
                ui.label(
                    s2kws_layout_job(balance.mature, network_type, theme_color().balance_syncing_color,FontId::proportional(28.))
                );
                ui.label(RichText::new(i18n("The balance may be out of date during node sync")).size(12.).color(theme_color().balance_syncing_color));
                return;
            } else {
                ui.label(
                    s2kws_layout_job(balance.mature, network_type, theme_color().balance_color,FontId::proportional(28.))
                );
            }

            if let Some(price_list) = core.market.price.as_ref() {
                for (symbol, data) in price_list.iter() {
                    if let Some(price) = data.price {
                        let text = format!("{:.8} {}", sompi_to_kaspa(balance.mature) * price, symbol.to_uppercase());
                        ui.label(RichText::new(text).font(FontId::proportional(16.)));
                    }
                }
            }

            if balance.pending != 0 {
                ui.label(format!(
                    "Pending: {}",
                    sompi_to_kaspa_string_with_suffix(
                        balance.pending,
                        network_type
                    )
                ));
            }
            if balance.outgoing != 0 {
                ui.label(format!(
                    "Sending: {}",
                    sompi_to_kaspa_string_with_suffix(
                        balance.outgoing,
                        network_type
                    )
                ));
            }

            ui.add_space(10.);

            let suffix = if balance.pending_utxo_count != 0 && balance.stasis_utxo_count != 0 {
                format!(" ({} pending, {} processing)", balance.pending_utxo_count, balance.stasis_utxo_count)
            } else if balance.pending_utxo_count != 0 {
                format!(" ({} pending)", balance.pending_utxo_count)
            } else if balance.stasis_utxo_count != 0 {
                format!(" ({} processing)", balance.stasis_utxo_count)
            } else {
                "".to_string()
            };

            if self.context.transaction_kind.is_none() {
                ui.label(format!(
                    "UTXOs: {}{suffix}",
                    balance.mature_utxo_count.separated_string(),
                ));
            }
        } else {
            ui.label("Balance: N/A");
        }


    }
}