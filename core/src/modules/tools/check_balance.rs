use crate::imports::*;

pub enum State {
    Select,
    CheckBalanceStart,
    CheckBalanceProcess,
    CheckBalanceResult(Result<u64>),
}

pub struct Tools {
    #[allow(dead_code)]
    runtime: Runtime,
    state: State,
    address_string: String,
    address: Option<Address>,
}

impl Tools {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            state: State::Select,
            address_string: Default::default(),
            address: None,
        }
    }
}

impl ModuleT for Tools {
    fn render(
        &mut self,
        core: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        match &self.state {
            State::Select => {
                ui.heading("Tools");
                ui.separator();
                ui.label("This is the tools page");
                if ui.button("Check Balance").clicked() {
                    self.state = State::CheckBalanceStart;
                }
            }
            State::CheckBalanceStart => {
                Panel::new(self)
                    .with_caption("Check Balance")
                    .with_back(|this| {
                        this.state = State::Select;
                    })
                    .with_close(|_| {
                        wallet.back();
                    })
                    .with_header(|_, ui| {
                        ui.label("Check Balance for a specific address");
                    })
                    .with_body(|this, ui| {
                        let address_string = this.address_string.clone();
                        ui.add(
                            TextEdit::singleline(&mut this.address_string)
                                .hint_text("Wallet Name...")
                                .vertical_align(Align::Center),
                        );

                        if address_string != this.address_string {
                            this.address_string = address_string.clone();
                            this.address = Address::try_from(address_string).ok();
                        }
                    })
                    .with_footer(|this, ui| {
                        if ui
                            .large_button_enabled(this.address.is_some(), "Check Balance")
                            .clicked()
                        {
                            this.state = State::CheckBalanceProcess;
                        }
                    })
                    .render(ui);
            }
            State::CheckBalanceProcess => {
                Panel::new(self)
                    .with_caption("Check Balance")
                    .with_back(|this| {
                        this.state = State::Select;
                    })
                    .with_close(|_| {
                        wallet.back();
                    })
                    .with_header(|_, ui| {
                        ui.label("Check Balance for a specific address");
                    })
                    .with_body(|this, _ui| {
                        let address_balance_result =
                            Payload::<Result<u64>>::new("tools_check_balance_result");
                        if !address_balance_result.is_pending() {
                            let _wallet = this.runtime.wallet().clone();
                            spawn_with_result(&address_balance_result, async move {
                                todo!("CheckBalanceProcess")
                            });
                        }

                        if let Some(balance_result) = address_balance_result.take() {
                            this.state = State::CheckBalanceResult(balance_result);
                        }
                    })
                    .render(ui);
            }
            State::CheckBalanceResult(_result) => {
                Panel::new(self)
                    .with_caption("Check Balance")
                    .with_back(|this| {
                        this.state = State::Select;
                    })
                    .with_close(|_| {
                        wallet.back();
                    })
                    .with_header(|_, ui| {
                        ui.label("Check Balance for a specific address");
                    })
                    .with_body(|_this, _ui| {
                        todo!("CheckBalanceResult");
                    })
                    .with_handler(|_this| {})
                    .render(ui);
            }
        }

        ui.heading("Request");
        ui.separator();
        ui.label("This is the payment request page");
    }
}
