use crate::imports::*;
use crate::primitives::account;
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

#[derive(Default)]
enum Details {
    #[default]
    Transactions,
    Account,
    UtxoSelector
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
enum Action {
    #[default]
    None,
    Estimating,
    Sending,
    // Reset,
    Processing,
}


impl Action {
    fn is_sending(&self) -> bool {
        matches!(self, Action::Sending | Action::Estimating | Action::Processing)
    }
}

#[derive(Default, Clone, Copy, Eq, PartialEq)]
enum TransactionKind {
    #[default]
    None,
    Send,
    Transfer,
    // Request,
}



#[derive(Default, Clone, Copy, Eq, PartialEq)]
enum Focus {
    #[default]
    None,
    Address,
    Amount,
    Fees,
    WalletSecret,
    PaymentSecret,
}

#[derive(Default)]
enum Status {
    #[default]
    None,
    GeneratorSummary(GeneratorSummary),
    Error(String),
    // Success(GeneratorSummary),
}

// impl Estimate {
//     fn is_ok(&self) -> bool {
//         matches!(self, Estimate::GeneratorSummary(_))
//     }

//     fn error(&mut self, error : impl Into<String>) {
//         *self = Estimate::Error(error.into());
//     }
// }

#[derive(Default, Clone, Eq, PartialEq)]
enum AddressStatus {
    #[default]
    None,
    Valid,
    NetworkMismatch(NetworkType),
    Invalid(String),
}

#[derive(Default)]
pub struct ManagerContext {
    selected: Option<Account>,
    details : Details,

    // send state
    destination_address_string : String,
    send_amount_text: String,
    send_amount_sompi : u64,
    enable_priority_fees : bool,
    priority_fees_text : String,
    priority_fees_sompi : u64,
    estimate : Arc<Mutex<Status>>,
    address_status : AddressStatus,
    action : Action,
    transaction_kind : TransactionKind,
    focus : Focus,
    wallet_secret : String,
    payment_secret : String,
}

impl ManagerContext {
    fn reset_send_state(&mut self) {
        
        println!("*** resetting send state...");

        self.destination_address_string = String::default();
        self.send_amount_text = String::default();
        self.send_amount_sompi = 0;
        self.enable_priority_fees = false;
        self.priority_fees_text = String::default();
        self.priority_fees_sompi = 0;
        *self.estimate.lock().unwrap() = Status::None;
        self.address_status = AddressStatus::None;
        self.action = Action::None;
        self.transaction_kind = TransactionKind::None;
        self.focus = Focus::None;
        self.wallet_secret.zeroize();
        self.payment_secret.zeroize();
    }
}

pub struct AccountManager {
    #[allow(dead_code)]
    runtime: Runtime,

    state: State,
    context : ManagerContext,
    editor_size : Vec2,
}

impl AccountManager {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            state: State::Select,
            context : ManagerContext::default(),
            editor_size : Vec2::INFINITY,
        }
    }

    pub fn select(&mut self, account: Option<Account>) {
        self.context.selected = account;
    }
}

impl ModuleT for AccountManager {

    fn reset(&mut self, _core: &mut Core) {
        self.context = ManagerContext::default();
        self.state = State::Select;
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        use egui_phosphor::light::{ARROW_CIRCLE_UP,ARROWS_DOWN_UP,QR_CODE};
        let screen_rect = ui.ctx().screen_rect();

        let network_type = if let Some(network_id) = core.state().network_id() {
            network_id.network_type()
        } else {
            core.settings.node.network.into()
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

            // State::Create => {

            //     //- ACCOUNT TYPE
            //     //- TODO   ACCOUNT NAME
            //     //- PROMPT FOR PASSWORD
            //     //- PAYMENT PASSWORD? 25th WORD?


            // }

            State::Overview { account } => {
                let width = ui.available_width();

                ui.horizontal(|ui| {

                    let wallet_name = if let Some(wallet_descriptor) = core.wallet_descriptor.as_ref() {
                        wallet_descriptor.title.as_deref().unwrap_or(wallet_descriptor.filename.as_str())
                    } else {
                        ui.label("Missing wallet descriptor");
                        return;
                    };

                    // let wallet_name = core.wallet_descriptor.as_ref().and_then(|descriptor|descriptor.title.clone()).as_deref().unwrap_or("NO NAME");
                    let current_wallet_selector_id = ui.make_persistent_id("current_wallet_selector");
                    let response = ui.add(Label::new(format!("Wallet: {} ⏷", wallet_name)).sense(Sense::click()));
                    
                    if response.clicked() {
                        ui.memory_mut(|mem| mem.toggle_popup(current_wallet_selector_id));
                    }
                    egui::popup::popup_above_or_below_widget(ui, current_wallet_selector_id, &response, AboveOrBelow::Below, |ui| {
                        ui.set_min_width(200.0);
                        ui.set_max_height(screen_rect.height() * 0.75);
                        ui.label("Select Wallet");
                        ui.label("");

                        ScrollArea::vertical()
                            .id_source("popup_wallet_selector_scroll")
                            .auto_shrink([true; 2])
                            .show(ui, |ui| {

                                let wallet_list = core.wallet_list().clone();

                                wallet_list.into_iter().for_each(|wallet_descriptor| {

                                    let title = if let Some(title) = wallet_descriptor.title.clone() {
                                        title
                                    } else if wallet_descriptor.filename.as_str() == "kaspa" {
                                        "Kaspa Wallet".to_string()
                                    } else {
                                        "NO NAME".to_string()
                                    };

                                    if ui.add(CompositeButton::new(
                                        title,
                                        wallet_descriptor.filename.clone(),
                                    )).clicked()
                                    {
                                        core.get_mut::<modules::WalletOpen>().open(wallet_descriptor.clone());
                                        core.select::<modules::WalletOpen>();
                                    }
                                });

                                ui.label("");
                                ui.separator();
                                ui.label("");
        
                                if ui.medium_button(
                                    "Create New Wallet",
                                ).clicked()
                                {
                                    core.select::<modules::WalletCreate>();
                                }

                            });


                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        
                        let current_account_selector_id = ui.make_persistent_id("current_account_selector");
                        let response = ui.add(Label::new(format!("Account: {} ⏷", account.name_or_id())).sense(Sense::click()));
                        
                        if response.clicked() {
                            ui.memory_mut(|mem| mem.toggle_popup(current_account_selector_id));
                        }
                        egui::popup::popup_above_or_below_widget(ui, current_account_selector_id, &response, AboveOrBelow::Below, |ui| {
                            ui.set_min_width(200.0);
                            ui.set_max_height(screen_rect.height() * 0.75);
                            ui.label("Select Account");
                            ui.label("");

                            egui::ScrollArea::vertical()
                                .id_source("popup_account_selector_scroll")
                                .auto_shrink([true; 2])
                                .show(ui, |ui| {
                
                                    if let Some(account_collection) = core.account_collection() {
                                        account_collection.iter().for_each(|account| {
                                            if ui
                                                .button(format!("Select {}\n{}", account.name_or_id(),account.balance().map(|balance|sompi_to_kaspa_string_with_suffix(balance.mature, &network_type)).unwrap_or("N/A".to_string())))
                                                .clicked()
                                            {
                                                self.state = State::Overview {
                                                    account: account.clone(),
                                                };
                                            }
                                        });

                                        ui.label("");
                                        ui.separator();
                                        ui.label("");
                                        use egui_phosphor::light::FOLDER_NOTCH_PLUS;
                                        if ui.medium_button(format!("{FOLDER_NOTCH_PLUS} Create New Account")).clicked() {
                                            core.select::<modules::AccountCreate>();
                                        }
                                    }

                                });

                        });
                        
                    });
                });

                SidePanel::left("account_manager_left").exact_width(width/2.).resizable(false).show_separator_line(true).show_inside(ui, |ui| {

                    ui.separator();
                    ui.add_space(8.);

                    egui::ScrollArea::vertical()
                        .id_source("overview_metrics")
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {

                            self.editor_size = Vec2::new(ui.available_width() * 0.75, 32.);

                            ui.vertical_centered(|ui| {

                                let account_context = if let Some(account_context) = account.context() {
                                    account_context
                                } else {
                                    ui.label("Account is missing context");
                                    return;
                                };

                                self.render_address(core, ui, &account, &account_context, network_type);

                                if !core.state().is_synced() || !core.state().is_connected() {
                                    self.render_network_state(core,ui);
                                    return;
                                }

                                self.render_balance(core, ui, &account, &account_context, network_type);

                                if self.context.action.is_sending() {
                                    self.render_send_ui(core, ui, &account, &account_context, network_type);
                                } else {
                                    
                                    self.render_qr(core, ui, &account_context);

                                    ui.vertical_centered(|ui|{
                                        ui.horizontal(|ui| {



                                            // if let Some(response) = 
                                            CenterLayoutBuilder::new()
                                                .add(Button::new(format!("{} Send", ARROW_CIRCLE_UP)).min_size(theme().medium_button_size()), |(this, _):&mut (&mut AccountManager, &mut Core)| {
                                                    this.context.action = Action::Estimating;
                                                    this.context.transaction_kind = TransactionKind::Send;
                                                })
                                                .add(Button::new(format!("{} Transfer", ARROWS_DOWN_UP)).min_size(theme().medium_button_size()), |(this,_)| {
                                                    this.context.action = Action::Estimating;
                                                    this.context.transaction_kind = TransactionKind::Transfer;
                                                })
                                                .add(Button::new(format!("{} Request", QR_CODE)).min_size(theme().medium_button_size()), |(_,core)| {
                                                    core.select::<modules::Request>();

                                                })
                                                .build(ui,&mut (self,core));
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
                                    self.context.details = Details::UtxoSelector;
                                }
                                ui.separator();
                                if ui.button("Details").clicked() {
                                    self.context.details = Details::Account;
                                }
                                ui.separator();
                                if ui.button("Transactions").clicked() {
                                    self.context.details = Details::Transactions;
                                }
                            });
                        });
                        ui.separator();

                        match self.context.details {
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

    fn render_network_state(&mut self, core : &mut Core, ui: &mut Ui) {
        use egui_phosphor::light::{CLOUD_SLASH,CLOUD_ARROW_DOWN};

        ui.vertical_centered(|ui|{
            ui.add_space(32.);
            if !core.state().is_connected() {
                ui.add_space(32.);
                ui.label(
                    RichText::new(CLOUD_SLASH)
                        .size(theme().icon_size_large)
                        .color(theme().icon_color_default)
                );
                ui.add_space(32.);
                
                ui.label("You are currently not connected to the Kaspa node.");
            } else if !core.state().is_synced() {
                
                ui.add_space(32.);
                ui.label(
                    RichText::new(CLOUD_ARROW_DOWN)
                        .size(theme().icon_size_large)
                        .color(theme().icon_color_default)
                );
                ui.add_space(32.);

                ui.label("The node is currently syncing with the Kaspa p2p network.");
                ui.add_space(16.);
                ui.label("Please wait for the node to sync or connect to a remote node.");
            }
            ui.add_space(32.);
            ui.label("You can configure a remote connection in Settings");
            ui.add_space(16.);
            if ui.large_button("Go to Settings").clicked() {
                core.select::<modules::Settings>();
            }
        });


    }

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
                    ui.add_space(8.);

                    let mut address_kind : Option<NewAddressKind> = None;
                    
                    ui.horizontal(|ui|{
                        if ui.medium_button("Generate New Receive Address").clicked() {
                            address_kind = Some(NewAddressKind::Receive);
                        }
                        if ui.medium_button("Generate New Change Address").clicked() {
                            address_kind = Some(NewAddressKind::Change);
                        }
                    });

                    if let Some(address_kind) = address_kind {
                        let account_id = account.id();
                        spawn(async move {
                            runtime()
                                .wallet()
                                .accounts_create_new_address(account_id, address_kind)
                                .await
                                .map_err(|err|Error::custom(format!("Failed to create new address\n{err}")))?;
                            // if let Err(err) = runtime().wallet().accounts_create_new_address(account_id, address_kind).await {
                            //     log_error!("Failed to create new address: {err}");
                            // }

                            runtime().request_repaint();

                            Ok(())
                        });
                    }
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

    fn render_address(&mut self, _core: &mut Core, ui : &mut Ui, _account : &Account, context : &account::AccountContext, _network_type : NetworkType) {
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
                runtime().notify(UserNotification::info("Address is copied to clipboard").short())
            }
    }

    fn render_balance(&mut self, _core: &mut Core, ui : &mut Ui, account : &Account, _context : &account::AccountContext, network_type : NetworkType) {

        // let theme = theme();

        ui.add_space(10.);

        if let Some(balance) = account.balance() {
            ui.heading(
                RichText::new(sompi_to_kaspa_string_with_suffix(balance.mature, &network_type)).font(FontId::proportional(28.)).color(theme().balance_color)
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

    fn render_qr(&mut self, _core: &mut Core, ui : &mut Ui, context : &account::AccountContext) {

        let scale = if self.context.action == Action::None { 1. } else { 0.35 };
        ui.add(
            egui::Image::new(ImageSource::Bytes { uri : Cow::Borrowed("bytes://qr.svg"), bytes: context.qr() })
            .fit_to_original_size(scale)
            .texture_options(TextureOptions::NEAREST)
        );

    }

    fn render_estimation_ui(&mut self, _core: &mut Core, ui: &mut egui::Ui, _account : &Account, _context : &Arc<account::AccountContext>, network_type: NetworkType) -> bool {
        use egui_phosphor::light::{CHECK, X};

        let mut request_estimate = false;

        TextEditor::new(
            &mut self.context.destination_address_string,
            // None,
            &mut self.context.focus,
            Focus::Address,
            |ui, text| {
                ui.add_space(8.);
                ui.label(egui::RichText::new("Enter destination address").size(12.).raised());
                ui.add_sized(self.editor_size, TextEdit::singleline(text)
                    .vertical_align(Align::Center))
            },
        )
        .change(|address| {
            match Address::try_from(address) {
                Ok(address) => {
                    let address_network_type = NetworkType::try_from(address.prefix).expect("prefix to network type");
                    if address_network_type != network_type {
                        self.context.address_status = AddressStatus::NetworkMismatch(address_network_type);
                    } else {
                        self.context.address_status = AddressStatus::Valid;
                    }
                }
                Err(err) => {
                    self.context.address_status = AddressStatus::Invalid(err.to_string());
                }
            }
        })
        .submit(|_, focus|{
            *focus = Focus::Amount;
        })
        .build(ui);
        
        match &self.context.address_status {
            AddressStatus::Valid => {},
            AddressStatus::None => {},
            AddressStatus::NetworkMismatch(address_network_type) => {
                ui.label(format!("This address if for the different\nnetwork ({address_network_type})"));
            },
            AddressStatus::Invalid(err) => {
                ui.label(format!("Please enter a valid address\n{err}"));
            }
        }

        let response = TextEditor::new(
            &mut self.context.send_amount_text,
            &mut self.context.focus,
            Focus::Amount,
            |ui, text| {
                ui.add_space(8.);
                ui.label(egui::RichText::new("Enter KAS amount to send").size(12.).raised());
                ui.add_sized(self.editor_size, TextEdit::singleline(text)
                    .vertical_align(Align::Center))
            },
        )
        .change(|_| {
            request_estimate = true;
        })
        .build(ui);

        if response.text_edit_submit(ui) {
            if self.context.enable_priority_fees {
                self.context.focus = Focus::Fees;
            } else if self.update_user_args() {
                self.context.action = Action::Sending;
            }
        }

        ui.add_space(8.);
        ui.checkbox(&mut self.context.enable_priority_fees,i18n("Include Priority Fees"));

        if self.context.enable_priority_fees {
            TextEditor::new(
                &mut self.context.priority_fees_text,
                &mut self.context.focus,
                Focus::Fees,
                |ui, text| {
                    ui.add_space(8.);
                    ui.label(egui::RichText::new("Enter priority fees").size(12.).raised());
                    ui.add_sized(self.editor_size, TextEdit::singleline(text)
                        .vertical_align(Align::Center))
                },
            )
            .change(|_| {
                request_estimate = true;
            })
            .submit(|_,_|{
                self.context.action = Action::Sending;
            })
            .build(ui); 
        }

        ui.add_space(8.);
        let ready_to_send = match &*self.context.estimate.lock().unwrap() {
            Status::GeneratorSummary(estimate) => {
                if let Some(final_transaction_amount) = estimate.final_transaction_amount {
                    ui.label(format!("Final Amount: {}", sompi_to_kaspa_string_with_suffix(final_transaction_amount + estimate.aggregated_fees, &network_type)));
                }
                ui.label(format!("Fees: {}", sompi_to_kaspa_string_with_suffix(estimate.aggregated_fees, &network_type)));
                ui.label(format!("Transactions: {} UTXOs: {}", estimate.number_of_generated_transactions, estimate.aggregated_utxos));
                
                self.context.address_status == AddressStatus::Valid
            }
            Status::Error(error) => {
                ui.label(RichText::new(error.to_string()).color(theme().error_color));
                false
            }
            Status::None => {
                ui.label("Please enter KAS amount to send");
                false
            }
        };
        ui.add_space(8.);

        ui.horizontal(|ui| {
            ui.vertical_centered(|ui|{
                ui.horizontal(|ui| {
                    CenterLayoutBuilder::new()
                        .add_enabled(ready_to_send, Button::new(format!("{CHECK} Send")).min_size(theme().medium_button_size()), |this: &mut AccountManager| {
                            this.context.action = Action::Sending;
                        })
                        .add(Button::new(format!("{X} Cancel")).min_size(theme().medium_button_size()), |this| {
                            this.context.reset_send_state();
                        })
                        .build(ui, self)
                });
            });

        });

        request_estimate
    }

    fn render_passphrase_ui(&mut self, _core: &mut Core, ui: &mut egui::Ui, account : &Account, _context : &Arc<account::AccountContext>, _network_type: NetworkType) -> bool {
        use egui_phosphor::light::{CHECK, X};

        let requires_payment_passphrase = account.requires_bip39_passphrase();
        let mut proceed_with_send = false;

        let response = TextEditor::new(
            &mut self.context.wallet_secret,
            &mut self.context.focus,
            Focus::WalletSecret,
            |ui, text| {
                ui.add_space(8.);
                ui.label(egui::RichText::new("Enter wallet password").size(12.).raised());
                ui.add_sized(self.editor_size, TextEdit::singleline(text)
                    .password(true)
                    .vertical_align(Align::Center))
            },
        )
        .build(ui);

        if response.text_edit_submit(ui) {
            if account.requires_bip39_passphrase() {
                self.context.focus = Focus::PaymentSecret;
            } else if !self.context.wallet_secret.is_empty() {
                proceed_with_send = true;
            }
        }

        if requires_payment_passphrase {
            let response = TextEditor::new(
                &mut self.context.payment_secret,
                &mut self.context.focus,
                Focus::WalletSecret,
                |ui, text| {
                    ui.add_space(8.);
                    ui.label(egui::RichText::new("Enter bip39 passphrase").size(12.).raised());
                    ui.add_sized(self.editor_size, TextEdit::singleline(text)
                        .password(true)
                        .vertical_align(Align::Center))
                },
            )
            .build(ui);

            if response.text_edit_submit(ui) && !self.context.wallet_secret.is_empty() && !self.context.payment_secret.is_empty() {
                proceed_with_send = true;
            }
    
        }

        let is_ready_to_send = !(self.context.wallet_secret.is_empty() || requires_payment_passphrase && self.context.payment_secret.is_empty());

        ui.add_space(8.);
        CenterLayoutBuilder::new()
            .add_enabled(is_ready_to_send, Button::new(format!("{CHECK} Submit")).min_size(theme().medium_button_size()), |_this: &mut AccountManager| {
                proceed_with_send = true;
            })
            .add(Button::new(format!("{X} Cancel")).min_size(theme().medium_button_size()), |this| {
                this.context.action = Action::Estimating;
            })
            .build(ui,self);



        proceed_with_send
    }

    fn render_send_ui(&mut self, core: &mut Core, ui: &mut egui::Ui, account : &Account, context : &Arc<account::AccountContext>, network_type: NetworkType) {

        ui.add_space(8.);
        ui.label("Sending funds");
        ui.add_space(8.);


        let send_result = Payload::<Result<GeneratorSummary>>::new("send_result");


        match self.context.action {
            Action::Estimating => {

                let request_estimate = self.render_estimation_ui(core, ui, account, context, network_type);

                if request_estimate && self.update_user_args() {

                    let priority_fees_sompi = if self.context.enable_priority_fees {
                        self.context.priority_fees_sompi
                    } else { 0 };

                    let address = match network_type {
                        NetworkType::Testnet => Address::try_from("kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya").unwrap(),
                        NetworkType::Mainnet => Address::try_from("kaspa:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqkx9awp4e").unwrap(),
                        _ => panic!("Unsupported network"),
                    };

                    let account_id = account.id();
                    let payment_output = PaymentOutput {
                        address,
                        amount: self.context.send_amount_sompi,
                    };

                    let status = self.context.estimate.clone();
                    spawn(async move {
                        let request = AccountsEstimateRequest {
                            task_id: None,
                            account_id,
                            destination: payment_output.into(),
                            priority_fee_sompi: Fees::SenderPaysAll(priority_fees_sompi),
                            payload: None,
                        };

                        match runtime().wallet().accounts_estimate_call(request).await {
                            Ok(response) => {
                                *status.lock().unwrap() = Status::GeneratorSummary(response.generator_summary);
                            }
                            Err(error) => {
                                *status.lock().unwrap() = Status::Error(error.to_string());
                            }    
                        }

                        runtime().egui_ctx().request_repaint();
                        Ok(())
                    });
                } 

            }

            Action::Sending => {

                let proceed_with_send = self.render_passphrase_ui(core, ui, account, context, network_type);

                if proceed_with_send {

                    let priority_fees_sompi = if self.context.enable_priority_fees {
                        self.context.priority_fees_sompi
                    } else { 0 };

                    let address = Address::try_from(self.context.destination_address_string.as_str()).expect("Invalid address");
                    let account_id = account.id();
                    let payment_output = PaymentOutput {
                        address,
                        amount: self.context.send_amount_sompi,
                    };
                    let wallet_secret = Secret::try_from(self.context.wallet_secret.clone()).expect("Invalid secret");
                    let payment_secret = None;

                    spawn_with_result(&send_result, async move {
                        let request = AccountsSendRequest {
                            // task_id: None,
                            account_id,
                            destination: payment_output.into(),
                            wallet_secret,
                            payment_secret,
                            priority_fee_sompi: Fees::SenderPaysAll(priority_fees_sompi),
                            payload: None,
                        };

                        let generator_summary = runtime().wallet().accounts_send_call(request).await?.generator_summary;
                        // let result = match runtime().wallet().accounts_send_call(request).await;
                        
                        //  {
                        //     Ok(_response) => {
                        //         // println!("RESPONSE: {:?}", response);
                        //         // *estimate.lock().unwrap() = Estimate::GeneratorSummary(response.generator_summary);
                        //     }
                        //     Err(error) => {
                        //         *status.lock().unwrap() = Status::Error(error.to_string());
                        //         // self.context.action = Action::Estimating;
                        //         // println!("ERROR: {}", error);
                        //         // *estimate.lock().unwrap() = Estimate::Error(error.to_string());
                        //     }    
                        // }

                        runtime().request_repaint();
                        Ok(generator_summary)
                    });
            
                    self.context.action = Action::Processing;
                }

            }
            Action::Processing => {
                ui.add_space(16.);
                ui.add(egui::Spinner::new().size(92.));

                if let Some(_result) = send_result.take() {

                    // - TODO - SET AND DISPLAY AN ERROR
                    // - PRESENT CLOSE BUTTON BEFORE CONTINUING
                    // - RESET STATE TO ESTIMATING?

                    self.context.action = Action::None;
                }
            }
            Action::None => {}
        }

    }

    fn update_user_args(&mut self) -> bool {
        let mut valid = true;

        match try_kaspa_str_to_sompi(self.context.send_amount_text.as_str()) {
            Ok(Some(sompi)) => {
                self.context.send_amount_sompi = sompi;
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

        match try_kaspa_str_to_sompi(self.context.priority_fees_text.as_str()) {
            Ok(Some(sompi)) => {
                self.context.priority_fees_sompi = sompi;
            }
            Ok(None) => {
                self.context.priority_fees_sompi = 0;
            }
            Err(err) => {
                self.user_error(format!("Invalid fee amount: {err}"));
                valid = false;
            }
        }

        valid
    }

    fn user_error(&self, error : impl Into<String>) {
        *self.context.estimate.lock().unwrap() = Status::Error(error.into());
    }

}