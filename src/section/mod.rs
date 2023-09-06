pub mod accounts;
pub mod deposit;
pub mod overview;
pub mod request;
pub mod send;
pub mod settings;
pub mod transactions;
pub mod unlock;

pub use accounts::Accounts;
pub use deposit::Deposit;
pub use overview::Overview;
pub use request::Request;
pub use send::Send;
pub use settings::Settings;
pub use transactions::Transactions;
pub use unlock::Unlock;

use crate::imports::*;

pub trait SectionT : Downcast {
    fn render(&mut self, _wallet : &mut Wallet, _ctx: &egui::Context, _frame: &mut eframe::Frame, _ui : &mut egui::Ui);
}

impl_downcast!(SectionT);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Section {
    Accounts,
    Deposit,
    Overview,
    Request,
    Send,
    Settings,
    Transactions,
    Unlock,
}

