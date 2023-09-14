use crate::imports::*;

#[allow(dead_code)]
enum State {
    Select,
    Overview { account: Arc<dyn runtime::Account> },
    Send { account: Arc<dyn runtime::Account> },
    Receive { account: Arc<dyn runtime::Account> },
}

pub struct Account {
    #[allow(dead_code)]
    interop: Interop,

    selected: Option<Arc<dyn runtime::Account>>,
    state: State,
}

impl Account {
    pub fn new(interop: Interop) -> Self {
        Self {
            interop,
            selected: None,
            state: State::Select,
        }
    }

    pub fn select(&mut self, account: Option<Arc<dyn runtime::Account>>) {
        self.selected = account;
    }
}

impl SectionT for Account {
    fn render(
        &mut self,
        wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        match &self.state {
            State::Select => {
                let accounts = wallet.account_list();

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

                if let Ok(network_id) = wallet.network_id() {
                    let network_type = network_id.network_type();

                    let address = account
                        .receive_address()
                        .map(String::from)
                        .unwrap_or(String::from("N/A"));
                    ui.label(format!("Address: {}", address));
                    // let balance = account.balance();
                    if let Some(balance) = account.balance() {
                        ui.label(format!(
                            "Balance: {}",
                            sompi_to_kaspa_string_with_suffix(balance.mature, &network_type)
                        ));
                        if balance.pending != 0 {
                            ui.label(format!(
                                "Pending: {}",
                                sompi_to_kaspa_string_with_suffix(balance.pending, &network_type)
                            ));
                        }
                    } else {
                        ui.label("Balance: N/A");
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
