use crate::imports::*;

#[allow(dead_code)]
enum State {
    Select,
    Overview,
    Send,
    Receive,
}

pub struct Account {
    #[allow(dead_code)]
    interop: Interop,

    selected: Option<Arc<dyn runtime::Account>>,
    state: State,
}

impl Account {
    pub fn new(interop: Interop) -> Self {
        Self {
            interop,
            selected: None,
            state: State::Overview,
        }
    }

    pub fn select(&mut self, account: Option<Arc<dyn runtime::Account>>) {
        self.selected = account;
    }
}

impl SectionT for Account {
    fn render(
        &mut self,
        wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        match self.state {
            State::Select => {
                let _accounts = wallet.account_list();
            }

            State::Overview => {
                ui.heading("Overview");
                ui.separator();
                ui.label("This is the overview page");
            }

            State::Send => {}

            State::Receive => {}
        }
    }
}
