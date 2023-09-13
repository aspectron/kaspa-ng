pub mod accounts;
pub mod create_wallet;
pub mod deposit;
pub mod import;
pub mod open;
pub mod overview;
pub mod request;
pub mod send;
pub mod settings;
pub mod transactions;

pub use accounts::Accounts;
pub use create_wallet::CreateWallet;
pub use deposit::Deposit;
pub use import::Import;
pub use open::Open;
pub use overview::Overview;
pub use request::Request;
pub use send::Send;
pub use settings::Settings;
pub use transactions::Transactions;

use crate::imports::*;

pub trait SectionT: Downcast {
    fn main(
        &mut self,
        _wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        _ui: &mut egui::Ui,
    );

    fn render(
        &mut self,
        wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        ui.style_mut().text_styles = wallet.large_style.text_styles.clone();
        self.main(wallet, _ctx, _frame, ui);
    }
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
