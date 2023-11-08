use std::borrow::Cow;

use crate::imports::*;

#[allow(dead_code)]
enum State {
    Select,
    Overview { account: Account },
    Send { account: Account },
    Receive { account: Account },
}

pub struct AccountManager {
    #[allow(dead_code)]
    interop: Interop,

    selected: Option<Account>,
    state: State,
}

impl AccountManager {
    pub fn new(interop: Interop) -> Self {
        Self {
            interop,
            selected: None,
            state: State::Select,
        }
    }

    pub fn select(&mut self, account: Option<Account>) {
        self.selected = account;
    }
}

impl ModuleT for AccountManager {
    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        let wallet_state = core.state();

        match &self.state {
            State::Select => {
                let accounts = core.account_list();

                if accounts.len() == 1 {
                    self.state = State::Overview {
                        account: accounts[0].clone(),
                    };
                } else {
                    ui.heading("Select Account");
                    ui.separator();

                    for account in accounts {
                        if ui
                            .button(format!("Select {}", account.name_or_id()))
                            .clicked()
                        {
                            self.state = State::Overview {
                                account: account.clone(),
                            };
                        }
                    }
                }
            }

            State::Overview { account } => {
                ui.heading("Overview");
                ui.separator();
                // ui.label("This is the overview page");
                ui.label(format!("Account: {}", account.name_or_id()));
                ui.label(" ");

                if let Some(network_id) = wallet_state.network_id() {
                    let network_type = network_id.network_type();

                    if let Some(context) = account.context() {
                        ui.label(format!("Address: {}", context.address()));

                        if ui.button("Copy to clipboard").clicked() {
                            ui.output_mut(|o| o.copied_text = context.address().to_string());
                        }

                        // let balance = account.balance();
                        if let Some(balance) = account.balance() {
                            ui.label(format!(
                                "Balance: {}",
                                sompi_to_kaspa_string_with_suffix(balance.mature, &network_type)
                            ));
                            if balance.pending != 0 {
                                ui.label(format!(
                                    "Pending: {}",
                                    sompi_to_kaspa_string_with_suffix(
                                        balance.pending,
                                        &network_type
                                    )
                                ));
                            }
                        } else {
                            ui.label("Balance: N/A");
                        }

                        ui.vertical_centered(|ui| {
                            ui.add(
                                egui::Image::new(ImageSource::Bytes { uri : Cow::Borrowed("bytes://qr.svg"), bytes: context.qr() })
                                .fit_to_original_size(1.)
                                // .shrink_to_fit()
                            );
                        });
                    } else {
                        ui.label("Account is missing context");
                    }
                } else {
                    ui.label("Network is not selected");
                    // TODO - GO TO SETTINGS
                }
            }

            State::Send { account: _ } => {}

            State::Receive { account: _ } => {}
        }
    }
}
