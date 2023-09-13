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

use std::any::type_name;

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
    fn render(
        &mut self,
        _wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        _ui: &mut egui::Ui,
    );

    // fn render(
    //     &mut self,
    //     wallet: &mut Wallet,
    //     _ctx: &egui::Context,
    //     _frame: &mut eframe::Frame,
    //     ui: &mut egui::Ui,
    // ) {
    //     ui.style_mut().text_styles = wallet.large_style.text_styles.clone();
    //     self.main(wallet, _ctx, _frame, ui);
    // }
}

impl_downcast!(SectionT);

pub struct Inner {
    pub name: String,
    pub type_name: String,
    pub type_id : TypeId,
    pub section: Rc<RefCell<dyn SectionT>>,

}

#[derive(Clone)]
pub struct Section {
    pub inner : Rc<Inner>,
}

impl Section {
    pub fn render(
        &self,
        wallet: &mut Wallet,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        self.inner.section.borrow_mut().render(wallet, ctx, frame, ui)
    }

    pub fn name(&self) -> &str {
        self.inner.name.as_str()
    }

    pub fn type_id(&self) -> TypeId {
        self.inner.type_id
    }
}

impl<T> From<Rc<RefCell<T>>> for Section
where
    T: SectionT + 'static,
{
    fn from(section: Rc<RefCell<T>>) -> Self {
        let type_name = type_name::<T>().to_string();
        let name = type_name.split("::").last().unwrap().to_string();
        let type_id = TypeId::of::<T>();
        Self { inner : Rc::new(Inner{ name, type_name, type_id, section }) }
    }
}

pub trait HashMapSectionExtension<T> {
    fn insert_typeid(&mut self, value: Rc<RefCell<T>>)
    where
        T: SectionT + 'static;
}

// impl<T> HashMapSectionExtension<T> for HashMap<TypeId, Rc<RefCell<dyn SectionT>>>
impl<T> HashMapSectionExtension<T> for HashMap<TypeId, Section>
where
    T: SectionT,
{
    fn insert_typeid(&mut self, section: Rc<RefCell<T>>) {
        // let name = type_name::<T>().to_string();
        self.insert(TypeId::of::<T>(), section.into());
    }
}
