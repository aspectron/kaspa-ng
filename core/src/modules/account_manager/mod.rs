use crate::imports::*;
use crate::primitives::account;
use std::borrow::Cow;
use kaspa_wallet_core::tx::{GeneratorSummary, PaymentOutput, Fees};
use kaspa_wallet_core::api::*;
use crate::primitives::descriptors::*;

mod overview;
mod transactions;
mod details;
mod utxo;
mod menus;

use overview::*;
use transactions::*;
use details::*;
use utxo::*;
use menus::*;


#[allow(dead_code)]
#[derive(Clone)]
enum AccountManagerState {
    Select,
    Overview { account: Account },
}

#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub enum AccountManagerSection {
    #[default]
    // None,
    Overview,
    Transactions,
    Details,
    UtxoManager
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
pub enum EstimatorStatus {
    #[default]
    None,
    GeneratorSummary(GeneratorSummary),
    Error(String),
}

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
    destination_address_string : String,
    send_amount_text: String,
    send_amount_sompi : u64,
    enable_priority_fees : bool,
    priority_fees_text : String,
    priority_fees_sompi : u64,
    estimate : Arc<Mutex<EstimatorStatus>>,
    address_status : AddressStatus,
    action : Action,
    transaction_kind : TransactionKind,
    focus : Focus,
    wallet_secret : String,
    payment_secret : String,
}

impl ManagerContext {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn reset_send_state(&mut self) {
        
        println!("*** resetting send state...");

        self.destination_address_string = String::default();
        self.send_amount_text = String::default();
        self.send_amount_sompi = 0;
        self.enable_priority_fees = false;
        self.priority_fees_text = String::default();
        self.priority_fees_sompi = 0;
        *self.estimate.lock().unwrap() = EstimatorStatus::None;
        self.address_status = AddressStatus::None;
        self.action = Action::None;
        self.transaction_kind = TransactionKind::None;
        self.focus = Focus::None;
        self.wallet_secret.zeroize();
        self.payment_secret.zeroize();
    }
}

pub struct RenderContext<'render> {
    pub account : &'render Account,
    pub context : Arc<account::AccountContext>,
    pub network_type : NetworkType,
    pub current_daa_score : Option<u64>,
}

impl<'render> RenderContext<'render> {
    pub fn new(account : &'render Account, network_type : NetworkType, current_daa_score : Option<u64>) -> Result<Self> {

        let context = if let Some(context) = account.context() {
            context
        } else {
            return Err(Error::custom("Account is missing context"));
        };

        Ok(Self {
            account,
            context,
            network_type,
            current_daa_score,
        })
    }
}

pub struct AccountManager {
    #[allow(dead_code)]
    runtime: Runtime,

    state: AccountManagerState,
    section: AccountManagerSection,
    context : ManagerContext,
}

impl AccountManager {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            state: AccountManagerState::Select,
            section: AccountManagerSection::Overview,
            context : ManagerContext::default(),
        }
    }
}

impl ModuleT for AccountManager {

    fn reset(&mut self, _core: &mut Core) {
        self.context = ManagerContext::default();
        self.state = AccountManagerState::Select;
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        if let Err(err) = self.render_state(core, ui) {
            ui.colored_label(theme().error_color, err.to_string());
        }
    }

}

impl AccountManager {

    pub fn select(&mut self, account: Option<Account>) {
        if let Some(account) = account {
            self.state = AccountManagerState::Overview {
                account: account.clone(),
            };
            
            if runtime().device().is_portrait() {
                self.section = AccountManagerSection::Overview;
            } else {
                self.section = AccountManagerSection::Transactions;
            }
        } else {
            self.state = AccountManagerState::Select;
        }

    }

    pub fn section(&mut self, section : AccountManagerSection) {
        self.section = section;
    }

    fn render_state(
        &mut self,
        core: &mut Core,
        ui: &mut egui::Ui,
    ) -> Result<()> {

        let network_type = if let Some(network_id) = core.state().network_id() {
            network_id.network_type()
        } else {
            core.settings.node.network.into()
        };

        let current_daa_score = core.state().current_daa_score();

        match self.state.clone() {
            AccountManagerState::Select => {

                if !core.state().is_open() {
                    core.select::<modules::WalletOpen>();
                } else if let Some(account_collection) = core.account_collection() {
                    if account_collection.is_empty() {
                        ui.label("Please create an account");
                    } else if account_collection.len() == 1 {
                        self.select(Some(account_collection.first().unwrap().clone()));
                        // self.state = AccountManagerState::Overview {
                        //     account: account_collection.first().unwrap().clone(),
                        // };
                    } else {
                        ui.heading("Select Account");
                        ui.separator();
    
                        account_collection.iter().for_each(|account| {
                            if ui
                                .button(format!("Select {}", account.name_or_id()))
                                .clicked()
                            {
                                self.select(Some(account.clone()));
                                if runtime().device().is_singular_layout() {
                                    self.section = AccountManagerSection::Overview;
                                } else {
                                    self.section = AccountManagerSection::Transactions;
                                }
                            }
                        });
                    }

                } else {
                    ui.label("Unable to access account list");
                }
            }

            AccountManagerState::Overview { account } => {
                let rc = RenderContext::new(&account, network_type, current_daa_score)?;
                // let section = self.section;
                if runtime().device().is_singular_layout() {
                    self.render_singular_layout(core,ui,&rc, self.section);
                } else {
                    if self.section == AccountManagerSection::Overview {
                        self.section = AccountManagerSection::Transactions;
                    }
                    self.render_landscape(core,ui,&rc, self.section);
                }
            }
        }

        Ok(())
    }


    fn render_menu(&mut self, core: &mut Core, ui: &mut Ui, rc : &RenderContext<'_>) {
        ui.horizontal(|ui| {
            let screen_rect_height = ui.ctx().screen_rect().height();
            WalletMenu::new().render(core,ui,screen_rect_height * 0.8);
            ui.separator();
            AccountMenu::new().render(core,ui,self,rc, screen_rect_height * 0.8);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ToolsMenu::new().render(core,ui,self, rc, screen_rect_height * 0.8);
            });
        });
    }

    fn render_landscape(&mut self, core: &mut Core, ui: &mut Ui, rc : &RenderContext<'_>, section : AccountManagerSection) {

        let panel_width = ui.available_width() * 0.5;

        self.render_menu(core,ui,rc);

        SidePanel::left("account_manager_left")
            .exact_width(panel_width)
            .resizable(false)
            .show_separator_line(true)
            // .frame(
            //     Frame::default()
            //         .inner_margin(0.)
            //         .outer_margin(4.)
            //         .fill(ui.ctx().style().visuals.panel_fill),
            // )
            .show_inside(ui, |ui| {
            Overview::new(&mut self.context).render(core,ui,rc);
        });
        
        SidePanel::right("account_manager_right")
            .exact_width(panel_width)
            .resizable(false)
            .show_separator_line(false)
            // .frame(
            //     Frame::default()
            //         .inner_margin(0.)
            //         .outer_margin(4.)
            //         .fill(ui.ctx().style().visuals.panel_fill),
            // )
            .show_inside(ui, |ui| {
                ui.separator();

                // ---
                ui.style_mut().text_styles = core.default_style.text_styles.clone();
                // ---

                egui::menu::bar(ui, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {

                        ui.add_space(32.);

                        if ui.button("UTXOs").clicked() {
                            self.section = AccountManagerSection::UtxoManager;
                        }
                        ui.separator();
                        if ui.button("Details").clicked() {
                            self.section = AccountManagerSection::Details;
                        }
                        ui.separator();
                        if ui.button("Transactions").clicked() {
                            self.section = AccountManagerSection::Transactions;
                        }
                    });
                });
                ui.separator();

                match section {
                    AccountManagerSection::Overview => {
                        Overview::new(&mut self.context).render(core,ui,rc);
                    }
                    AccountManagerSection::Transactions => {
                        Transactions::new().render(ui,core,rc);
                    }
                    AccountManagerSection::Details => {
                        Details::new().render(core,ui,rc);
                    }
                    AccountManagerSection::UtxoManager => {
                        UtxoManager::new().render(core,ui,rc);
                    }
                }
            });


    }

    fn render_singular_layout(&mut self, core: &mut Core, ui: &mut Ui, rc : &RenderContext<'_>, section : AccountManagerSection) {

        match section {
            AccountManagerSection::Overview => {
                Overview::new(&mut self.context).render(core,ui,rc);
            }
            AccountManagerSection::Transactions => {
                Transactions::new().render(ui,core,rc);
            }
            AccountManagerSection::Details => {
                Details::new().render(core,ui,rc);
            }
            AccountManagerSection::UtxoManager => {
                UtxoManager::new().render(core,ui,rc);
            }
        }

    }

}