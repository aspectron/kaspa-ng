use crate::imports::*;
use egui_phosphor::thin::*;
use kaspa_wallet_core::{wallet::Wallet, account::{BIP32_ACCOUNT_KIND, LEGACY_ACCOUNT_KIND}};

#[derive(Clone)]
pub enum State {
    Select,
    Settings { account : Account },
    WalletSecret { account : Account },
    Spawn { account : Account },
    Status,
    Finish,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
enum Focus {
    #[default]
    None,
    WalletSecret,
}

#[derive(Default, Clone)]
enum Status {
    #[default]
    None,
    Error { message : String },
    Processing { index : usize, utxo_count: usize, balance : u64}
}

impl Status {
    fn error(message : &str) -> Self {
        Self::Error { message : message.to_string() }
    }

    fn processing(index : usize, utxo_count: usize, balance : u64) -> Self {
        Self::Processing { index, utxo_count, balance }
    }
}

#[derive(Default)]
struct ScannerContext {
    transfer_funds : bool,
    wallet_secret: String,
    status : Arc<Mutex<Status>>,
    abortable : Abortable,
}

impl Zeroize for ScannerContext {
    fn zeroize(&mut self) {
        self.wallet_secret.zeroize();
        self.status = Arc::new(Mutex::new(Status::default()));
        self.abortable.reset();
    }
}

pub struct Scanner {
    #[allow(dead_code)]
    runtime: Runtime,
    context: ScannerContext,
    state: State,
    focus: FocusManager<Focus>,
}

impl Scanner {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
            context: ScannerContext::default(),
            state: State::Select,
            focus: FocusManager::default(),
        }
    }
}

impl ModuleT for Scanner {

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Mobile
    }

    fn modal(&self) -> bool {
        true
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        let network_type = if let Some(network_id) = core.state().network_id() {
            network_id.network_type()
        } else {
            core.settings.node.network.into()
        };

        match self.state.clone() {

            State::Select => {

                let back = Rc::new(RefCell::new(false));

                Panel::new(self)
                    .with_caption(i18n("Scanner"))
                    .with_back(|_this| {
                        *back.borrow_mut() = true;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_this,ui| {
                        if core.state().is_open() && core.state().is_connected() && core.state().is_synced() {
                            ui.label(i18n("Please select account to scan"));
                        }
                    })
                    .with_body(|this, ui|{

                        if !core.state().is_open() {
                            ui.label(
                                RichText::new(SEAL_WARNING)
                                    .size(theme_style().icon_size_large)
                                    .color(theme_color().error_color)
                            );
                            ui.add_space(8.);                                    
                            ui.label(i18n("Scanner requires an open wallet."));
                            ui.add_space(16.);

                            if ui.large_button(i18n("Close")).clicked() {
                                *back.borrow_mut() = true;
                            }
                            
                            return;
                        } else if !core.state().is_connected() {
                            ui.label(
                                RichText::new(CLOUD_X)
                                    .size(theme_style().icon_size_large)
                                    .color(theme_color().error_color)
                            );
                            ui.add_space(8.);                                    
                            ui.label(i18n("You are currently not connected to the Kaspa node."));
                            ui.add_space(16.);
                            
                            if ui.large_button(i18n("Close")).clicked() {
                                *back.borrow_mut() = true;
                            }

                            return;
                        } else if !core.state().is_synced() {
                            ui.label(
                                RichText::new(CLOUD_ARROW_DOWN)
                                    .size(theme_style().icon_size_medium)
                                    .color(theme_color().warning_color)
                            );
                            ui.add_space(8.);
                            ui.label(i18n("The node is currently syncing with the Kaspa p2p network. Please wait for the node to sync."));
                            ui.add_space(16.);

                            if ui.large_button(i18n("Close")).clicked() {
                                *back.borrow_mut() = true;
                            }

                            return;
                        }

                        if let Some(account_collection) = core.account_collection() {

                            for selectable_account in account_collection.list() {

                                match selectable_account.account_kind().as_ref() {
                                    BIP32_ACCOUNT_KIND | LEGACY_ACCOUNT_KIND => {
                                        if ui.account_selector_button(selectable_account, &network_type, false, core.balance_padding()).clicked() {
                                            this.state = State::Settings {
                                                account: selectable_account.clone(),
                                            };
                                        }
                                    }
                                    _ => {},
                                }

                            }
                        }

                    }).render(ui);

                    if *back.borrow() {
                        if core.has_stack() {
                            core.back();
                        } else {
                            core.select::<modules::Overview>();
                        }
                    }
            }
            State::Settings { account } => {

                Panel::new(self)
                    .with_caption(i18n("Settings"))
                    .with_back(|this| {
                        this.state = State::Select;
                    })
                    .with_header(|_ctx,_ui| {
                        // ui.label("Please enter the wallet secret");
                    })
                    .with_body(|this,ui| {

                        ui.checkbox(&mut this.context.transfer_funds, i18n("Transfer funds during scan"));

                        ui.label("");
                        ui.label(i18n("This option will transfer any discovered funds to the first change address of this account."));

                    })
                    .with_footer(|this,ui| {
                        if ui.large_button(i18n("Continue")).clicked() {
                            this.state = State::WalletSecret { account };
                            this.focus.next(Focus::WalletSecret)
                        }
                    })
                    .render(ui);
            }
            State::WalletSecret { account } => {

                let submit = Rc::new(RefCell::new(false));

                Panel::new(self)
                    .with_caption(i18n("Wallet Secret"))
                    .with_back(|this| {
                        this.state = State::Select;
                    })
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.label(i18n("Please enter the wallet secret"));
                    })
                    .with_body(|this,ui| {
                        TextEditor::new(
                            &mut this.context.wallet_secret,
                            &mut this.focus,
                            Focus::WalletSecret,
                            |ui, text| {
                                ui.label(RichText::new(i18n("Enter your wallet secret")).size(12.).raised());
                                ui.add_sized(theme_style().panel_editor_size, TextEdit::singleline(text)
                                    .vertical_align(Align::Center)
                                    .password(true))
                            },
                        ).submit(|text,_focus| {
                            if !text.is_empty() {
                                *submit.borrow_mut() = true;
                            }
                        })
                        .build(ui);
                    })
                    .with_footer(|this,ui| {
                        let enabled = !this.context.wallet_secret.is_empty();
                        if ui.large_button_enabled(enabled,i18n("Continue")).clicked() {
                            *submit.borrow_mut() = true;
                        }
                    })
                    .render(ui);

                if *submit.borrow() {
                    self.state = State::Spawn { account };
                    self.focus.next(Focus::None);
                }

            }
            State::Spawn { account } => {

                if let Ok(wallet) = core.wallet().downcast_arc::<Wallet>() {
                    let status = self.context.status.clone();
                    let abortable = self.context.abortable.clone();
                    let wallet_secret = Secret::from(self.context.wallet_secret.as_str());
                    let transfer_funds = self.context.transfer_funds;
                    self.context.wallet_secret.zeroize();
                    spawn(async move {
                        let binding = wallet.guard();
                        let guard = binding.lock().await;
                        if let Some(account) = wallet.get_account_by_id(&account.id(),&guard).await? {
                            account.as_derivation_capable()?
                                .derivation_scan(
                                    wallet_secret,
                                    None,
                                    0,
                                    usize::MAX,
                                    64,
                                    transfer_funds,
                                    None,
                                    &abortable,
                                    true,
                                    Some(Arc::new(move |index,utxo_count, balance, txid|{
                                        if let Some(_txid) = txid {
                                            // println!("txid: {}", txid);
                                            // println!("scanner - txid: {}, balance: {}", txid, balance);
                                        } else {
                                            *status.lock().unwrap() = Status::processing(index, utxo_count, balance);
                                        }
                                    }))
                                ).await?;

                        } else {
                            *status.lock().unwrap() = Status::error(i18n("Account not found"));
                        }

                        Ok(())
                    });

                    self.state = State::Status;

                } else {
                    ui.label("");
                    ui.label(i18n("Unable to access the wallet subsystem"));
                    ui.label("");
                    if ui.large_button(i18n("Continue")).clicked() {
                        self.state = State::Select;
                    }
                }
            }
            State::Status => {

                Panel::new(self)
                    .with_caption(i18n("Scanner"))
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.label(i18n("Processing..."));
                    })
                    .with_body(|this,ui| {

                        match &*this.context.status.lock().unwrap() {
                            Status::Error { message } => {
                                ui.label(message);
                            }
                            Status::Processing { index, utxo_count, balance } => {
                                //ui.label(format!("Scanning address derivation {}...", index.separated_string()));
                                //ui.label(format!("Located {} UTXOs", utxo_count.separated_string()));
                                ui.label(i18n_args("Scanning address derivation {index}...", &[("index", &index.separated_string())]));
                                ui.label(i18n_args("Located {utxo_count} UTXOs", &[("utxo_count", &utxo_count.separated_string())]));

                                ui.add_space(16.);
                                ui.label(RichText::new(i18n("BALANCE")).size(12.).raised());
                                ui.label(
                                    s2kws_layout_job(core.balance_padding(), *balance, &network_type, theme_color().balance_color,FontId::proportional(28.))
                                );
                            }
                            _ => {}
                        }

                        // ui.label("");
                        ui.add_space(64.);
                        ui.add(egui::Spinner::new().size(92.));

                    })
                    .with_footer(|this,ui| {
                        if ui.large_button(i18n("Stop")).clicked() {
                            this.context.abortable.abort();
                            this.state = State::Finish;
                        }
                    })
                    .render(ui);

            }

            State::Finish => {

                let balance_padding = core.balance_padding();

                Panel::new(self)
                    .with_caption(i18n("Scanner"))
                    .with_close_enabled(false, |_|{
                    })
                    .with_header(|_ctx,ui| {
                        ui.label(i18n("Scanning complete..."));
                    })
                    .with_body(|this,ui| {

                        if let Status::Processing { index,utxo_count, balance } = &*this.context.status.lock().unwrap() {
                            //ui.label(format!("Total addresses scanned: {}", index.separated_string()));
                            //ui.label(format!("Located {} UTXOs", utxo_count.separated_string()));
                            ui.label(i18n_args("Total addresses scanned: {index}", &[("index", &index.separated_string())]));
                            ui.label(i18n_args("Located {utxo_count} UTXOs", &[("utxo_count", &utxo_count.separated_string())]));
                            ui.add_space(16.);
                            ui.label(RichText::new(i18n("BALANCE")).size(12.).raised());
                            ui.label(
                                s2kws_layout_job(balance_padding, *balance, &network_type, theme_color().balance_color,FontId::proportional(28.))
                            );
                        }

                    })
                    .with_footer(|this,ui| {
                        if ui.large_button(i18n("Close")).clicked() {
                            this.context.zeroize();
                            this.state = State::Select;
                            core.select::<modules::AccountManager>();
                        }
                    })
                    .render(ui);
            }
        }
    }
}
