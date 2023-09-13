pub mod accounts;
pub mod deposit;
pub mod open;
pub mod overview;
pub mod request;
pub mod send;
pub mod settings;
pub mod transactions;
pub mod create_wallet;
pub mod import;

pub use accounts::Accounts;
pub use deposit::Deposit;
pub use open::Open;
pub use overview::Overview;
pub use request::Request;
pub use send::Send;
pub use settings::Settings;
pub use transactions::Transactions;
pub use create_wallet::CreateWallet;
pub use import::Import;

use crate::imports::*;

pub trait SectionT: Downcast {
    fn render(
        &mut self,
        _wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        _ui: &mut egui::Ui,
    );
}

impl_downcast!(SectionT);

pub trait HashMapSectionExtension<T> {
    fn insert_typeid(&mut self, value: Rc<RefCell<T>>)
    where
        T: SectionT + 'static;
}

impl<T> HashMapSectionExtension<T> for HashMap<TypeId, Rc<RefCell<dyn SectionT>>>
where
    T: SectionT,
{
    fn insert_typeid(&mut self, value: Rc<RefCell<T>>) {
        self.insert(TypeId::of::<T>(), value);
    }
}
