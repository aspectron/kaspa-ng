use crate::imports::*;
use super::*;

pub struct BalancePane<'context> {
    context : &'context ManagerContext,
}

impl<'context> BalancePane<'context> {
    
    pub fn new(context : &'context ManagerContext) -> Self {
        Self { context }
    }
    
    pub fn render(&mut self, core: &mut Core, ui : &mut Ui, rc : &RenderContext) {
    
        let RenderContext { account, network_type, .. } = rc;

        ui.add_space(10.);

        if let Some(balance) = account.balance() {
            
            if !core.state().is_synced() {
                ui.label(
                    s2kws_layout_job(core.balance_padding(), balance.mature, network_type, theme_color().balance_syncing_color,FontId::proportional(28.))
                );
                ui.label(RichText::new(i18n("The balance may be out of date during node sync")).size(12.).color(theme_color().balance_syncing_color));
                return;
            } else {
                ui.label(
                    s2kws_layout_job(core.balance_padding(), balance.mature, network_type, theme_color().balance_color,FontId::proportional(28.))
                );
            }

            if core.settings.market_monitor && (core.settings.node.network == Network::Mainnet || core.settings.developer.market_monitor_on_testnet) {
                if let Some(market) = core.market.as_ref() {
                    if let Some(price_list) = market.price.as_ref() {
                        let mut symbols = price_list.keys().collect::<Vec<_>>();
                        symbols.sort();
                        ui.vertical_centered(|ui| {
                            let text = symbols.into_iter().filter_map(|symbol| {
                                    price_list.get(symbol).map(|data| {
                                        let symbol = symbol.to_uppercase();
                                        let MarketData { price,  precision, .. } = data;
                                        // let text = 
                                        let amount = sompi_to_kaspa(balance.mature) * (*price);
                                        format_currency_with_symbol(amount, *precision, symbol.as_str())
                                    })
                            }).collect::<Vec<_>>().join("  ");
                            ui.label(RichText::new(text).font(FontId::proportional(14.)));
                        });
                    }
                }
            }
            
            if balance.pending != 0 {
                ui.label(i18n_args(
                    "Pending: {amount}",
                    &[("amount", &sompi_to_kaspa_string_with_suffix(
                        balance.pending,
                        network_type
                    ))]
                ));
            }
            if balance.outgoing != 0 {
                ui.label(i18n_args(
                    "Sending: {amount}",
                    &[("amount", &sompi_to_kaspa_string_with_suffix(
                        balance.outgoing,
                        network_type
                    ))]
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
            ui.label(i18n("Balance: N/A"));
        }


    }
}