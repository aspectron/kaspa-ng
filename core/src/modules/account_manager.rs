use crate::imports::*;

use std::borrow::Cow;
use kaspa_wallet_core::tx::{GeneratorSummary, PaymentOutput, Fees};
use kaspa_wallet_core::api::*;

#[allow(dead_code)]
#[derive(Clone)]
enum State {
    Select,
    Overview { account: Account },
    Send { account: Account },
    Receive { account: Account },
}

#[derive(Default)]
enum Estimate {
    #[default]
    None,
    GeneratorSummary(GeneratorSummary),
    Error(String),
}

pub struct AccountManager {
    #[allow(dead_code)]
    interop: Interop,

    selected: Option<Account>,
    state: State,
    send_amount_text: String,
    send_amount_sompi : u64,
    send_info: Option<String>,
    // running_estimate : bool,
    estimate : Arc<Mutex<Estimate>>,
}

impl AccountManager {
    pub fn new(interop: Interop) -> Self {
        Self {
            interop,
            selected: None,
            state: State::Select,
            send_amount_text: String::new(),
            send_amount_sompi : 0,
            send_info : None,
            estimate : Arc::new(Mutex::new(Estimate::None)),
            // running_estimate : false,
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

        match self.state.clone() {
            State::Select => {
                if let Some(account_collection) = core.account_collection() {
                    if account_collection.is_empty() {
                        ui.label("Please create an account");
                    } else if account_collection.len() == 1 {
                        self.state = State::Overview {
                            account: account_collection.first().unwrap().clone(),
                        };
                    } else {
                        ui.heading("Select Account");
                        ui.separator();
    
                        // for account in account_collection.iter() {
                        account_collection.iter().for_each(|account| {
                            if ui
                                .button(format!("Select {}", account.name_or_id()))
                                .clicked()
                            {
                                self.state = State::Overview {
                                    account: account.clone(),
                                };
                            }
                        });
                    }

                } else {
                    ui.label("Unable to access account list");
                }

            }

            State::Overview { account } => {
                ui.heading("Wallet");
                // ui.label("This is the overview page");
                ui.label(format!("Account: {}", account.name_or_id()));
                ui.separator();
                ui.label(" ");

                let network_type = if let Some(network_id) = wallet_state.network_id() {
                    network_id.network_type()
                } else {
                    ui.label("Network is not selected");
                    return;
                };

                let context = if let Some(context) = account.context() {
                    context
                } else {
                    ui.label("Account is missing context");
                    return;
                };

                ui.label(format!("Address: {}", context.address()));

                if ui.button(RichText::new(egui_phosphor::light::CLIPBOARD_TEXT)).clicked() {
                    ui.output_mut(|o| o.copied_text = context.address().to_string());
                }

                // let balance = account.balance();
                if let Some(balance) = account.balance() {
                    // ui.label("Balance");
                    ui.heading(
                        RichText::new(sompi_to_kaspa_string_with_suffix(balance.mature, &network_type)).code()
                    );
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

                if let Some((mature_utxo_size, pending_utxo_size)) =
                    account.utxo_sizes()
                {
                    if pending_utxo_size == 0 {
                        ui.label(format!(
                            "UTXOs: {}",
                            mature_utxo_size,
                        ));
                    } else {
                        ui.label(format!(
                            "UTXOs: {} ({} pending)",
                            mature_utxo_size, pending_utxo_size
                        ));
                    }
                } else {
                    ui.label("No UTXOs");
                }

                ui.vertical_centered(|ui| {
                    ui.add(
                        egui::Image::new(ImageSource::Bytes { uri : Cow::Borrowed("bytes://qr.svg"), bytes: context.qr() })
                        .fit_to_original_size(1.)
                        // .shrink_to_fit()
                    );
                });
                
                ui.horizontal(|ui| {
                    if ui.button("Send").clicked() {
                        self.state = State::Send {
                            account: account.clone(),
                        };
                    }
                    if ui.button("Receive").clicked() {
                        self.state = State::Receive {
                            account: account.clone(),
                        };
                    }
                });

                ui.separator();
                
                // -----------------------------------------------------------------
                // -----------------------------------------------------------------
                // -----------------------------------------------------------------

                let size = egui::Vec2::new(300_f32, 32_f32);

                ui.label(egui::RichText::new("Enter amount to send").size(12.).raised());
                let mut send_amount_text = self.send_amount_text.clone();
                ui.add_sized(
                    size,
                    TextEdit::singleline(&mut send_amount_text)
                        // .hint_text("Payment password...")
                        .vertical_align(Align::Center),
                );

                if let Some(send_info) = &self.send_info {
                    ui.label(send_info);
                }



                if send_amount_text != self.send_amount_text {
                    self.send_amount_text = send_amount_text;
                    match try_kaspa_str_to_sompi(self.send_amount_text.clone()) {
                        Ok(Some(send_amount_sompi)) => {
                            self.send_info = None;
                            self.send_amount_sompi = send_amount_sompi;

                            // let account = self.selected.clone().unwrap();
                            // self.selected()
                            // - TODO -
                            let address = Address::try_from(context.address()).expect("Invalid address");

                            // pub fn update_wallet_list(&self) {
                            let interop = self.interop.clone();
                            let account_id = account.id(); //self.selected.clone().unwrap().id();
                            // let payment_destination = self.selected.clone().unwrap().context().unwrap().address().clone();
                            let payment_output = PaymentOutput {
                                address,
                                amount: self.send_amount_sompi,
                            };

                            let estimate = self.estimate.clone();

                            spawn(async move {
                                let request = AccountEstimateRequest {
                                    task_id: None,
                                    account_id,
                                    destination: payment_output.into(),
                                    priority_fee_sompi: Fees::SenderPaysAll(0),
                                    payload: None,
                                };

                                match interop.wallet().account_estimate_call(request).await {
                                    Ok(response) => {
                                        *estimate.lock().unwrap() = Estimate::GeneratorSummary(response.generator_summary);
                                    }
                                    Err(error) => {
                                        *estimate.lock().unwrap() = Estimate::Error(error.to_string());
                                    }    
                                }

                                interop.egui_ctx().request_repaint();
                                Ok(())
                            });

                        }
                        Ok(None) => {
                            self.send_info = None;
                            *self.estimate.lock().unwrap() = Estimate::None;
                        }
                        Err(_) => {
                            *self.estimate.lock().unwrap() = Estimate::None;
                            self.send_info = Some("Please enter amount".to_string());
                        }
                    }
                }

                match &*self.estimate.lock().unwrap() {
                    // pub network_type: NetworkType,
                    // pub aggregated_utxos: usize,
                    // pub aggregated_fees: u64,
                    // pub number_of_generated_transactions: usize,
                    // pub final_transaction_amount: Option<u64>,
                    // pub final_transaction_id: Option<TransactionId>,
                    Estimate::GeneratorSummary(estimate) => {
                        if let Some(final_transaction_amount) = estimate.final_transaction_amount {
                            ui.label(format!("Final Amount: {}", sompi_to_kaspa_string_with_suffix(final_transaction_amount + estimate.aggregated_fees, &network_type)));
                        }
                        ui.label(format!("Fees: {}", sompi_to_kaspa_string_with_suffix(estimate.aggregated_fees, &network_type)));
                        ui.label(format!("Transactions: {} UTXOs: {}", estimate.number_of_generated_transactions, estimate.aggregated_utxos));
                        // ui.label(format!("Transactions: {}", estimate.number_of_generated_transactions));
                    }
                    Estimate::Error(error) => {
                        ui.label(RichText::new(error.to_string()).color(theme().error_color));
                    }
                    Estimate::None => {
                        ui.label("Please enter KAS amount to send");
                    }
                    // ui.label(format!("Estimated size: {}", estimate.size));
                    // ui.label(format!("Estimated total: {}", sompi_to_kaspa_string_with_suffix(estimate.total, &network_type)));
                }

                // let account_send_estimate_result = Payload::<Result<Arc<GeneratorSummary>>>::new("account_send_estimate");
                // if !account_send_estimate_result.is_pending() {
                // }

                // if let Some(result) = account_send_estimate_result.take() {
                //     match result {
                //         Ok(estimate_result) => {
                //             // println!("Account created successfully");
                //             // self.state = State::PresentMnemonic(creation_data);
                //             // wallet.get_mut::<section::Account>().select(Some(creation_dataaccount));
                //         }
                //         Err(err) => {
                //             // println!("Account creation error: {}", err);
                //             // self.state = State::AccountError(Arc::new(err));
                //         }
                //     }
                // }

                // -----------------------------------------------------------------
                // -----------------------------------------------------------------
                // -----------------------------------------------------------------

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {

                    let transactions = account.transactions();

                    if transactions.is_empty() {
                        ui.label("No transactions");
                    } else {
                        transactions.reverse_iter().for_each(|transaction| {
                            transaction.render(ui, network_type, wallet_state.current_daa_score(), false);
                        });
                    }

                });


            }

            State::Send { account: _ } => {}

            State::Receive { account: _ } => {}
        }
    }
}
