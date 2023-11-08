use std::any::type_name;

use crate::imports::*;

kaspa_ng_macros::register_modules!([
    account_manager,
    deposit,
    metrics,
    wallet_open,
    request,
    send,
    settings,
    transactions,
    account_create,
    wallet_create,
    export,
    import,
    testing,
    logs,
    changelog,
    welcome,
    overview,
    node,
]);

pub enum ModuleStyle {
    Large,
    Default,
}

pub enum ModuleCaps {
    Desktop,
    Mobile,
    WebApp,
    Extension,
}

pub trait ModuleT: Downcast {
    fn name(&self) -> Option<&'static str> {
        None
    }

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Large
    }

    fn init(&mut self, _wallet: &mut Core) {}

    fn render(
        &mut self,
        _wallet: &mut Core,
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

impl_downcast!(ModuleT);

pub struct Inner {
    pub name: String,
    pub type_name: String,
    pub type_id: TypeId,
    pub module: Rc<RefCell<dyn ModuleT>>,
}

#[derive(Clone)]
pub struct Module {
    pub inner: Rc<Inner>,
}

impl Module {
    pub fn init(&self, wallet: &mut Core) {
        self.inner.module.borrow_mut().init(wallet)
    }

    pub fn render(
        &self,
        wallet: &mut Core,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        let mut module = self.inner.module.borrow_mut();

        match module.style() {
            ModuleStyle::Large => {
                ui.style_mut().text_styles = wallet.large_style.text_styles.clone();
            }
            ModuleStyle::Default => {
                ui.style_mut().text_styles = wallet.default_style.text_styles.clone();
            }
        }

        module.render(wallet, ctx, frame, ui)
    }

    pub fn name(&self) -> &str {
        self.inner
            .module
            .borrow_mut()
            .name()
            .unwrap_or_else(|| self.inner.name.as_str())
        // self.inner.name.as_str()
    }

    pub fn type_id(&self) -> TypeId {
        self.inner.type_id
    }

    pub fn get<M>(&self) -> Ref<'_, M>
    where
        M: ModuleT + 'static,
    {
        Ref::map(self.inner.module.borrow(), |r| {
            (r).as_any()
                .downcast_ref::<M>()
                .expect("unable to downcast section")
        })
    }

    pub fn get_mut<M>(&mut self) -> RefMut<'_, M>
    where
        M: ModuleT + 'static,
    {
        RefMut::map(self.inner.module.borrow_mut(), |r| {
            (r).as_any_mut()
                .downcast_mut::<M>()
                .expect("unable to downcast_mut module")
        })
    }
}

impl<T> From<Rc<RefCell<T>>> for Module
where
    T: ModuleT + 'static,
{
    fn from(section: Rc<RefCell<T>>) -> Self {
        let type_name = type_name::<T>().to_string();
        let name = type_name.split("::").last().unwrap().to_string();
        let type_id = TypeId::of::<T>();
        Self {
            inner: Rc::new(Inner {
                name,
                type_name,
                type_id,
                module: section,
            }),
        }
    }
}

pub trait HashMapModuleExtension<T> {
    fn insert_typeid(&mut self, value: T)
    where
        T: ModuleT + 'static;

    // fn get_with_typeid<M>(&self) -> Ref<'_, M>
    // where
    //     M: ModuleT + 'static;

    // fn get_mut_with_typeid<M>(&mut self) -> RefMut<'_, M>
    // where
    //     M: ModuleT + 'static;
}

// impl<T> HashMapSectionExtension<T> for HashMap<TypeId, Rc<RefCell<dyn SectionT>>>
impl<T> HashMapModuleExtension<T> for HashMap<TypeId, Module>
where
    T: ModuleT,
{
    fn insert_typeid(&mut self, section: T) {
        let section = Rc::new(RefCell::new(section));
        // let name = type_name::<T>().to_string();
        self.insert(TypeId::of::<T>(), section.into());
    }

    // fn get_with_typeid<M>(&self) -> Ref<'_, M>
    // where
    //     M: ModuleT + 'static,
    // {
    //     let cell = self.get(&TypeId::of::<M>()).unwrap();
    //     Ref::map(cell.inner.module.borrow(), |r| {
    //         (r).as_any()
    //             .downcast_ref::<M>()
    //             .expect("unable to downcast section")
    //     })
    // }

    // fn get_mut_with_typeid<M>(&mut self) -> RefMut<'_, M>
    // where
    //     M: ModuleT + 'static,
    // {
    //     let cell = self.get_mut(&TypeId::of::<M>()).unwrap();
    //     RefMut::map(cell.inner.module.borrow_mut(), |r| {
    //         (r).as_any_mut()
    //             .downcast_mut::<M>()
    //             .expect("unable to downcast_mut module")
    //     })
    // }
}
