use crate::imports::*;
use crate::primitives::account::Context;
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

#[derive(Clone, Eq, PartialEq)]
enum Action {
    None,
    Sending,
    Estimating,
}

impl Action {
    // fn is_none(&self) -> bool {
    //     matches!(self, Action::None)
    // }

    fn is_sending(&self) -> bool {
        matches!(self, Action::Sending | Action::Estimating)
    }
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
    runtime: Runtime,

    selected: Option<Account>,
    state: State,
    destination_address_string : String,
    send_amount_text: String,
    send_amount_sompi : u64,
    send_info: Option<String>,
    // running_estimate : bool,
    estimate : Arc<Mutex<Estimate>>,
    action : Action,
    wallet_secret : String,
    payment_secret : String,
}

impl AccountManager {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            selected: None,
            state: State::Select,
            destination_address_string : String::new(),
            send_amount_text: String::new(),
            send_amount_sompi : 0,
            send_info : None,
            estimate : Arc::new(Mutex::new(Estimate::None)),
            action : Action::None,
            wallet_secret : String::new(),
            payment_secret : String::new(),
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

        // let wallet_state = core.state();
        let network_type = if let Some(network_id) = core.state().network_id() {
            network_id.network_type()
        } else {
            ui.label("Network is not selected");
            return;
        };

        let current_daa_score = core.state().current_daa_score();

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
                let width = ui.available_width();

                ui.horizontal(|ui| {

                    ui.heading("Wallet");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("Account: {}", account.name_or_id()));
                        
                    });
                });




                SidePanel::left("account_manager_left").exact_width(width/2.).resizable(false).show_separator_line(true).show_inside(ui, |ui| {
                    // ui.label("Kaspa NG");
                    egui::ScrollArea::vertical()
                        .id_source("overview_metrics")
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
        
                            ui.vertical_centered(|ui| {


                                // ui.label("This is the overview page");
                                let context = if let Some(context) = account.context() {
                                    context
                                } else {
                                    ui.label("Account is missing context");
                                    return;
                                };
                                ui.separator();
                                // ui.label(" ");
                                ui.add_space(8.);

                                ui.horizontal(|ui|{

                                    let address = format_address(context.address(), Some(8));
                                    // ui.label(format!("Address: {}", context.address()));
                                    ui.label(format!("Address: {address}"));
                                    if ui.button(RichText::new(egui_phosphor::light::CLIPBOARD_TEXT)).clicked() {
                                        ui.output_mut(|o| o.copied_text = context.address().to_string());
                                    }
                                });


                                // let network_type = if let Some(network_id) = wallet_state.network_id() {
                                //     network_id.network_type()
                                // } else {
                                //     ui.label("Network is not selected");
                                //     return;
                                // };




                                // let balance = account.balance();
                                if let Some(balance) = account.balance() {
                                    // ui.label("Balance");
                                    ui.heading(
                                        RichText::new(sompi_to_kaspa_string_with_suffix(balance.mature, &network_type)).font(FontId::proportional(24.))
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
                                    if balance.outgoing != 0 {
                                        ui.label(format!(
                                            "Sending: {}",
                                            sompi_to_kaspa_string_with_suffix(
                                                balance.outgoing,
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

                                    ui.add(
                                        egui::Image::new(ImageSource::Bytes { uri : Cow::Borrowed("bytes://qr.svg"), bytes: context.qr() })
                                        .fit_to_original_size(1.)
                                        .texture_options(TextureOptions::NEAREST)
                                        // .shrink_to_fit()
                                    );
                                // });
                                
                                // ui.separator();


                                
                                // -----------------------------------------------------------------
                                // -----------------------------------------------------------------
                                // -----------------------------------------------------------------

                                if self.action.is_sending() {
                                    self.render_send_ui(core, ui, &account, &context, network_type);
                                } else {
                                    ui.vertical_centered(|ui|{

                                        ui.horizontal(|ui| {
                                            if ui.medium_button(format!("{} Send", egui_phosphor::light::ARROW_CIRCLE_UP)).clicked() {
                                                self.action = Action::Estimating;
                                                // self.state = State::Send {
                                                //     account: account.clone(),
                                                // };
                                            }
                                            if ui.medium_button(format!("{} Request", egui_phosphor::light::QR_CODE)).clicked() {
                                                // self.state = State::Receive {
                                                //     account: account.clone(),
                                                // };
                                            }
                                        });

                                    });

                                }
                            });
                                // -----------------------------------------------------------------
                                // -----------------------------------------------------------------
                                // -----------------------------------------------------------------
                        });
                    });

                
                SidePanel::right("account_manager_right")
                    .exact_width(width/2.)
                    .resizable(false)
                    .show_separator_line(false)
                    .show_inside(ui, |ui| {    
                        ui.separator();

                        egui::ScrollArea::vertical().auto_shrink([false,false]).show(ui, |ui| {

                            let transactions = account.transactions();

                            if transactions.is_empty() {
                                ui.label("No transactions");
                            } else {
                                let total: u64 = transactions.iter().map(|transaction|transaction.aggregate_input_value()).sum();
                                transactions.iter().for_each(|transaction| {
                                    transaction.render(ui, network_type, current_daa_score, true, Some(total));
                                });
                            }

                        });
                    });

            }

            State::Send { account: _ } => {}

            State::Receive { account: _ } => {}
        }
    }
}

impl AccountManager {
    fn render_send_ui(&mut self, _core: &mut Core, ui: &mut egui::Ui, account : &Account, _context : &Arc<Context>, network_type: NetworkType) {


        let size = egui::Vec2::new(300_f32, 32_f32);

        let mut proceed_with_estimate = false;

        let mut destination_address_string = self.destination_address_string.clone();
        ui.label(egui::RichText::new("Enter address").size(12.).raised());

        // TODO - address processing...
        let _response = ui.add_sized(
            size,
            TextEdit::singleline(&mut destination_address_string)
                // .hint_text("Payment password...")
                .vertical_align(Align::Center),
        );
        if destination_address_string != self.destination_address_string {
            self.destination_address_string = destination_address_string;
            match try_user_string_to_address(self.destination_address_string.as_str(), &network_type) {
                Ok(_address) => {},
                Err(err) => {
                    self.send_info = Some(err.to_string());
                }
            }
        }

        ui.label(egui::RichText::new("Enter amount to send").size(12.).raised());
        let mut send_amount_text = self.send_amount_text.clone();
        let response = ui.add_sized(
            size,
            TextEdit::singleline(&mut send_amount_text)
                // .hint_text("Payment password...")
                .vertical_align(Align::Center),
        );
        if response.text_edit_submit(ui) {
            proceed_with_estimate = true;
        } else if self.action == Action::Estimating {
            response.request_focus();
        }

        if let Some(send_info) = &self.send_info {
            ui.label(send_info);
        }

        if send_amount_text != self.send_amount_text {
            self.send_amount_text = send_amount_text;
            match try_kaspa_str_to_sompi(self.send_amount_text.clone()) {
                Ok(Some(send_amount_sompi)) => {
                    self.send_info = None;
                    self.send_amount_sompi = send_amount_sompi;

                    // - TODO -
                    let address = Address::try_from("kaspatest:qqz22l98sf8jun72rwh5rqe2tm8lhwtdxdmynrz4ypwak427qed5juktjt7ju").expect("Invalid address");
                    // let address = Address::try_from(context.address()).expect("Invalid address");
                    let runtime = self.runtime.clone();
                    let account_id = account.id();
                    let payment_output = PaymentOutput {
                        address,
                        amount: self.send_amount_sompi,
                    };

                    let estimate = self.estimate.clone();

                    spawn(async move {
                        let request = AccountsEstimateRequest {
                            task_id: None,
                            account_id,
                            destination: payment_output.into(),
                            priority_fee_sompi: Fees::SenderPaysAll(0),
                            payload: None,
                        };

                        match runtime.wallet().accounts_estimate_call(request).await {
                            Ok(response) => {
                                *estimate.lock().unwrap() = Estimate::GeneratorSummary(response.generator_summary);
                            }
                            Err(error) => {
                                *estimate.lock().unwrap() = Estimate::Error(error.to_string());
                            }    
                        }

                        runtime.egui_ctx().request_repaint();
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
            Estimate::GeneratorSummary(estimate) => {
                if let Some(final_transaction_amount) = estimate.final_transaction_amount {
                    ui.label(format!("Final Amount: {}", sompi_to_kaspa_string_with_suffix(final_transaction_amount + estimate.aggregated_fees, &network_type)));
                }
                ui.label(format!("Fees: {}", sompi_to_kaspa_string_with_suffix(estimate.aggregated_fees, &network_type)));
                ui.label(format!("Transactions: {} UTXOs: {}", estimate.number_of_generated_transactions, estimate.aggregated_utxos));
            }
            Estimate::Error(error) => {
                ui.label(RichText::new(error.to_string()).color(theme().error_color));
            }
            Estimate::None => {
                ui.label("Please enter KAS amount to send");
            }
        }



        match self.action {
            Action::Estimating => {
                



                ui.horizontal(|ui| {
                    if ui.medium_button_enabled(!self.send_amount_text.is_empty() && self.send_amount_sompi > 0,"Send").clicked() {
                        proceed_with_estimate = true;
                    }
                    if proceed_with_estimate {
                        self.action = Action::Sending;
                    }

                    if ui.medium_button("Cancel").clicked() {
                        self.reset();
                        // *self.estimate.lock().unwrap() = Estimate::None;
                        // self.send_amount_text = String::new();
                        // self.action = Action::None;
                    }
                });
            }

            Action::Sending => {
                ui.label(egui::RichText::new("Enter wallet password").size(12.).raised());

                let mut proceed_with_send = false;
                // let mut send_amount_text = self.send_amount_text.clone();
                let response = ui.add_sized(
                    size,
                    TextEdit::singleline(&mut self.wallet_secret)
                        // .hint_text("Payment password...")
                        .password(true)
                        .vertical_align(Align::Center),
                );
                if response.text_edit_submit(ui) {
                    proceed_with_send = true;
                } else {
                    response.request_focus();
                }

                ui.horizontal(|ui| {

                    if ui.medium_button_enabled(!self.wallet_secret.is_empty(),"Send").clicked() {
                        proceed_with_send = true;
                    }

                    if proceed_with_send {

                        // let address = Address::try_from(context.address()).expect("Invalid address");
                        let address = Address::try_from("kaspatest:qqz22l98sf8jun72rwh5rqe2tm8lhwtdxdmynrz4ypwak427qed5juktjt7ju").expect("Invalid address");
                        let runtime = self.runtime.clone();
                        let account_id = account.id();
                        let payment_output = PaymentOutput {
                            address,
                            amount: self.send_amount_sompi,
                        };
                        let wallet_secret = Secret::try_from(self.wallet_secret.clone()).expect("Invalid secret");
                        let payment_secret = None; // Secret::try_from(self.payment_secret.clone()).expect("Invalid secret");
    
                        spawn(async move {
                            let request = AccountsSendRequest {
                                task_id: None,
                                account_id,
                                destination: payment_output.into(),
                                wallet_secret,
                                payment_secret,
                                priority_fee_sompi: Fees::SenderPaysAll(0),
                                payload: None,
                            };
    
                            match runtime.wallet().accounts_send_call(request).await {
                                Ok(response) => {
                                    println!("****** RESPONSE: {:?}", response);
                                    // *estimate.lock().unwrap() = Estimate::GeneratorSummary(response.generator_summary);
                                }
                                Err(error) => {
                                    println!("****** ERROR: {}", error);
                                    // *estimate.lock().unwrap() = Estimate::Error(error.to_string());
                                }    
                            }
    
                            runtime.egui_ctx().request_repaint();
                            Ok(())
                        });
                
                        self.reset();
                    }
                    if ui.medium_button("Cancel").clicked() {

                        self.reset();
                    }
                });

            }
            _=>{}
        }

    }

    fn reset(&mut self) {
        *self.estimate.lock().unwrap() = Estimate::None;
        self.send_amount_text = String::new();
        self.send_amount_sompi = 0;
        self.action = Action::None;
        self.wallet_secret.zeroize();
        self.payment_secret.zeroize();
}
}