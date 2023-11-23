use crate::imports::*;
use crate::primitives::account::Context;
use std::borrow::Cow;
use kaspa_wallet_core::tx::{GeneratorSummary, PaymentOutput, Fees};
use kaspa_wallet_core::api::*;
use crate::primitives::descriptors::*;

#[allow(dead_code)]
#[derive(Clone)]
enum State {
    Select,
    Overview { account: Account },
    Send { account: Account },
    Receive { account: Account },
}

enum Details {
    Transactions,
    Account,
    UtxoSelector
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Action {
    None,
    Estimating,
    Sending,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Focus {
    None,
    Address,
    Amount,
    Fees,
}

impl Action {
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

// impl Estimate {
//     fn is_ok(&self) -> bool {
//         matches!(self, Estimate::GeneratorSummary(_))
//     }

//     fn error(&mut self, error : impl Into<String>) {
//         *self = Estimate::Error(error.into());
//     }
// }

#[derive(Clone, Eq, PartialEq)]
enum AddressStatus {
    Valid,
    None,
    NetworkMismatch(NetworkType),
    Invalid(String),
}

pub struct AccountManager {
    #[allow(dead_code)]
    runtime: Runtime,

    selected: Option<Account>,
    state: State,
    details : Details,
    destination_address_string : String,
    send_amount_text: String,
    send_amount_sompi : u64,
    enable_priority_fees : bool,
    priority_fees_text : String,
    priority_fees_sompi : u64,
    send_info: Option<String>,
    estimate : Arc<Mutex<Estimate>>,
    address_status : AddressStatus,
    action : Action,
    focus : Focus,
    wallet_secret : String,
    payment_secret : String,
}

impl AccountManager {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            selected: None,
            state: State::Select,
            details : Details::Transactions,
            destination_address_string : String::new(),
            send_amount_text: String::new(),
            send_amount_sompi : 0,
            enable_priority_fees : false,
            priority_fees_text : String::new(),
            priority_fees_sompi : 0,
            send_info : None,
            estimate : Arc::new(Mutex::new(Estimate::None)),
            address_status : AddressStatus::None,
            action : Action::None,
            focus : Focus::None,
            wallet_secret : String::new(),
            payment_secret : String::new(),
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
        use egui_phosphor::light::{ARROW_CIRCLE_UP,QR_CODE};

        let theme = theme();

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

                    ui.heading("Kaspa Wallet");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("Account: {}", account.name_or_id()));
                        
                    });
                });

                SidePanel::left("account_manager_left").exact_width(width/2.).resizable(false).show_separator_line(true).show_inside(ui, |ui| {

                    ui.separator();
                    ui.add_space(8.);

                    egui::ScrollArea::vertical()
                        .id_source("overview_metrics")
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
        
                            ui.vertical_centered(|ui| {

                                let context = if let Some(context) = account.context() {
                                    context
                                } else {
                                    ui.label("Account is missing context");
                                    return;
                                };

                                self.render_balance(core, ui, &account, &context, network_type);

                                if self.action.is_sending() {
                                    self.render_send_ui(core, ui, &account, &context, network_type);
                                } else {
                                    
                                    self.render_qr(core, ui, &context);

                                    ui.vertical_centered(|ui|{
                                        ui.horizontal(|ui| {
                                            CenterLayoutBuilder::new()
                                                .add(Button::new(format!("{} Send", ARROW_CIRCLE_UP)).min_size(theme.medium_button_size()), || {
                                                    self.action = Action::Estimating;
                                                })
                                                .add(Button::new(format!("{} Request", QR_CODE)).min_size(theme.medium_button_size()), || {})
                                                .build(ui);
                                        });
                                    });

                                }
                            });
                        });
                    });
                
                SidePanel::right("account_manager_right")
                    .exact_width(width/2.)
                    .resizable(false)
                    .show_separator_line(false)
                    .show_inside(ui, |ui| {    
                        ui.separator();

                        // ---
                        ui.style_mut().text_styles = core.default_style.text_styles.clone();
                        // ---

                        egui::menu::bar(ui, |ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {

                                ui.add_space(32.);

                                if ui.button("UTXOs").clicked() {
                                    self.details = Details::UtxoSelector;
                                }
                                ui.separator();
                                if ui.button("Details").clicked() {
                                    self.details = Details::Account;
                                }
                                ui.separator();
                                if ui.button("Transactions").clicked() {
                                    self.details = Details::Transactions;
                                }
                            });
                        });
                        ui.separator();

                        match self.details {
                            Details::Transactions => {
                                self.render_transactions(ui, core, &account, network_type, current_daa_score);
                            }
                            Details::Account => {
                                self.render_account_details(ui, core, &account);
                            }
                            Details::UtxoSelector => {
                                self.render_utxo_selector(ui, core, &account);
                            }
                        }
                    });
            }

            State::Send { account: _ } => {}

            State::Receive { account: _ } => {}
        }
    }
}

impl AccountManager {

    fn render_transactions(&mut self, ui: &mut Ui, _core : &mut Core, account : &Account, network_type : NetworkType, current_daa_score : Option<u64>) {
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
    }

    fn render_account_details(&mut self, ui: &mut Ui, _core : &mut Core, account : &Account) {
        egui::ScrollArea::vertical().auto_shrink([false,false]).show(ui, |ui| {

            let descriptor = account.descriptor();

            match &*descriptor {
                AccountDescriptor::Bip32(descriptor) => {
                    descriptor.render(ui);
                },
                _ => {
                    ui.label("Unknown descriptor type");
                }
            }
        });
    }

    fn render_utxo_selector(&mut self, ui: &mut Ui, _core : &mut Core, _account : &Account) {
        egui::ScrollArea::vertical().auto_shrink([false,false]).show(ui, |ui| {
            ui.label("UTXO Selection");
        });

    }

    fn render_balance(&mut self, _core: &mut Core, ui : &mut Ui, account : &Account, context : &Context, network_type : NetworkType) {

        let theme = theme();

        use egui_phosphor::light::CLIPBOARD_TEXT;
        let address = format_address(context.address(), Some(8));
        if ui.add(Label::new(format!("Address: {address} {CLIPBOARD_TEXT}")).sense(Sense::click()))
            // .on_hover_ui_at_pointer(|ui|{
            //     ui.vertical(|ui|{
            //         ui.add(Label::new(format!("{}", context.address().to_string())));
            //         ui.add_space(16.);
            //         ui.label("Click to copy address to clipboard".to_string());
            //     });
            // })
            .clicked() {
                ui.output_mut(|o| o.copied_text = context.address().to_string());
            }
        ui.add_space(10.);

        if let Some(balance) = account.balance() {
            ui.heading(
                RichText::new(sompi_to_kaspa_string_with_suffix(balance.mature, &network_type)).font(FontId::proportional(28.)).color(theme.balance_color)
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

        ui.add_space(10.);


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

    }

    fn render_qr(&mut self, _core: &mut Core, ui : &mut Ui, context : &Context) {

        let scale = if self.action == Action::None { 1. } else { 0.35 };
        ui.add(
            egui::Image::new(ImageSource::Bytes { uri : Cow::Borrowed("bytes://qr.svg"), bytes: context.qr() })
            .fit_to_original_size(scale)
            .texture_options(TextureOptions::NEAREST)
        );

    }

    fn render_send_ui(&mut self, _core: &mut Core, ui: &mut egui::Ui, account : &Account, _context : &Arc<Context>, network_type: NetworkType) {

        let theme = theme();
        let size = egui::Vec2::new(300_f32, 32_f32);
        let mut request_estimate = false;

        ui.add_space(8.);
        ui.label("Sending funds");
        ui.add_space(8.);

        TextEditor::new(
            &mut self.destination_address_string,
            // None,
            &mut self.focus,
            Focus::Address,
            |ui, text| {
                ui.label(egui::RichText::new("Enter destination address").size(12.).raised());
                ui.add_sized(size, TextEdit::singleline(text)
                    .vertical_align(Align::Center))
            },
        )
        .change(|address| {
            match Address::try_from(address) {
                Ok(address) => {
                    let address_network_type = NetworkType::try_from(address.prefix).expect("prefix to network type");
                    if address_network_type != network_type {
                        self.address_status = AddressStatus::NetworkMismatch(address_network_type);
                    } else {
                        self.address_status = AddressStatus::Valid;
                    }
                }
                Err(err) => {
                    self.address_status = AddressStatus::Invalid(err.to_string());
                }
            }
        })
        .submit(|_, focus|{
            *focus = Focus::Amount;
        })
        .build(ui);
        
        match &self.address_status {
            AddressStatus::Valid => {},
            AddressStatus::None => {},
            AddressStatus::NetworkMismatch(address_network_type) => {
                ui.label(format!("This address if for the different\nnetwork ({address_network_type})"));
            },
            AddressStatus::Invalid(err) => {
                ui.label(format!("Please enter a valid address\n{err}"));
            }
        }

        TextEditor::new(
            &mut self.send_amount_text,
            &mut self.focus,
            Focus::Amount,
            |ui, text| {
                ui.label(egui::RichText::new("Enter KAS amount to send").size(12.).raised());
                ui.add_sized(size, TextEdit::singleline(text)
                    .vertical_align(Align::Center))
            },
        )
        .change(|_| {
            request_estimate = true;
        })
        .submit(|_, focus|{
            if self.enable_priority_fees {
                *focus = Focus::Fees;
            } else {
                self.action = Action::Sending;
            }
        })
        .build(ui); 

        ui.checkbox(&mut self.enable_priority_fees,i18n("Include Priority Fees"));

        if self.enable_priority_fees {
            TextEditor::new(
                &mut self.priority_fees_text,
                &mut self.focus,
                Focus::Fees,
                |ui, text| {
                    ui.label(egui::RichText::new("Enter priority fees").size(12.).raised());
                    ui.add_sized(size, TextEdit::singleline(text)
                        .vertical_align(Align::Center))
                },
            )
            .change(|_| {
                request_estimate = true;
            })
            .submit(|_,_|{
                self.action = Action::Sending;
            })
            .build(ui); 
        }

        if let Some(send_info) = &self.send_info {
            ui.label(send_info);
        }

        match self.action {
            Action::Estimating => {
                // if request_estimate {
                //     println!("request estimate: {}", request_estimate);
                // }

                if request_estimate && self.update_estimate_args() {

                    self.send_info = None;
                    // self.send_amount_sompi = send_amount_sompi;
                    let priority_fees_sompi = if self.enable_priority_fees {
                        self.priority_fees_sompi
                    } else { 0 };

                    let address = match network_type {
                        NetworkType::Testnet => Address::try_from("kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya").unwrap(),
                        NetworkType::Mainnet => Address::try_from("kaspa:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqkx9awp4e").unwrap(),
                        _ => panic!("Unsupported network"),
                    };

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
                            priority_fee_sompi: Fees::SenderPaysAll(priority_fees_sompi),
                            payload: None,
                        };

                        match runtime.wallet().accounts_estimate_call(request).await {
                            Ok(response) => {
                                println!("estimate ok");
                                *estimate.lock().unwrap() = Estimate::GeneratorSummary(response.generator_summary);
                            }
                            Err(error) => {
                                println!("estimate error");
                                *estimate.lock().unwrap() = Estimate::Error(error.to_string());
                            }    
                        }

                        runtime.egui_ctx().request_repaint();
                        Ok(())
                    });
                } 

                let ready_to_send = match &*self.estimate.lock().unwrap() {
                    Estimate::GeneratorSummary(estimate) => {
                        println!("rendering estimate...");
                        if let Some(final_transaction_amount) = estimate.final_transaction_amount {
                            ui.label(format!("Final Amount: {}", sompi_to_kaspa_string_with_suffix(final_transaction_amount + estimate.aggregated_fees, &network_type)));
                        }
                        ui.label(format!("Fees: {}", sompi_to_kaspa_string_with_suffix(estimate.aggregated_fees, &network_type)));
                        ui.label(format!("Transactions: {} UTXOs: {}", estimate.number_of_generated_transactions, estimate.aggregated_utxos));
                        
                        self.address_status == AddressStatus::Valid
                    }
                    Estimate::Error(error) => {
                        ui.label(RichText::new(error.to_string()).color(theme.error_color));
                        false
                    }
                    Estimate::None => {
                        ui.label("Please enter KAS amount to send");
                        false
                    }
                };

                ui.horizontal(|ui| {
                    use egui_phosphor::light::{CHECK, X};
                    ui.vertical_centered(|ui|{
                        ui.horizontal(|ui| {
                            let mut reset = false;
                            CenterLayoutBuilder::new()
                                .add_enabled(ready_to_send, Button::new(format!("{CHECK} Send")).min_size(theme.medium_button_size()), || {
                                    self.action = Action::Sending;
                                })
                                .add(Button::new(format!("{X} Cancel")).min_size(theme.medium_button_size()), || {
                                    reset = true;
                                })
                                .build(ui);

                            if reset {
                                self.reset();
                            }
                        });
                    });

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

                        let priority_fees_sompi = if self.enable_priority_fees {
                            self.priority_fees_sompi
                        } else { 0 };
    
                        let address = Address::try_from(self.destination_address_string.as_str()).expect("Invalid address");
                        let runtime = self.runtime.clone();
                        let account_id = account.id();
                        let payment_output = PaymentOutput {
                            address,
                            amount: self.send_amount_sompi,
                        };
                        let wallet_secret = Secret::try_from(self.wallet_secret.clone()).expect("Invalid secret");
                        let payment_secret = None;
    
                        spawn(async move {
                            let request = AccountsSendRequest {
                                task_id: None,
                                account_id,
                                destination: payment_output.into(),
                                wallet_secret,
                                payment_secret,
                                priority_fee_sompi: Fees::SenderPaysAll(priority_fees_sompi),
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

    fn update_estimate_args(&mut self) -> bool {
        let mut valid = true;

        match try_kaspa_str_to_sompi(self.send_amount_text.as_str()) {
            Ok(Some(sompi)) => {
                self.send_amount_sompi = sompi;
            }
            Ok(None) => {
                self.user_error("Please enter an amount".to_string());
                valid = false;
            }
            Err(err) => {
                self.user_error(format!("Invalid amount: {err}"));
                valid = false;
            }
        }

        match try_kaspa_str_to_sompi(self.priority_fees_text.as_str()) {
            Ok(Some(sompi)) => {
                self.priority_fees_sompi = sompi;
            }
            Ok(None) => {
                self.priority_fees_sompi = 0;
            }
            Err(err) => {
                self.user_error(format!("Invalid fee amount: {err}"));
                valid = false;
            }
        }

        valid
    }

    fn user_error(&self, error : impl Into<String>) {
        *self.estimate.lock().unwrap() = Estimate::Error(error.into());
    }

    fn reset(&mut self) {
        *self.estimate.lock().unwrap() = Estimate::None;
        self.address_status = AddressStatus::None;
        self.destination_address_string = String::default();
        self.send_amount_text = String::default();
        self.send_amount_sompi = 0;
        self.action = Action::None;
        self.focus = Focus::None;
        self.wallet_secret.zeroize();
        self.payment_secret.zeroize();
    }

}