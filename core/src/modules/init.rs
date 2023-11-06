use crate::imports::*;

pub struct Init {
    #[allow(dead_code)]
    interop: Interop,
    account : Option<Arc<dyn runtime::Account>>,
}

impl Init {
    pub fn new(interop: Interop) -> Self {
        Self { interop, account : None }
    }

    pub fn select(&mut self, account : Option<Arc<dyn runtime::Account>>) {
        self.account = account;
    }
}

impl ModuleT for Init {
    fn render(
        &mut self,
        _wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        _ui: &mut egui::Ui,
    ) {




    }
}
