use crate::imports::*;
use crate::interop::Interop;
use crate::section::HashMapSectionExtension;
use crate::sync::SyncStatus;
use kaspa_wallet_core::events::Events as CoreWallet;
use kaspa_wallet_core::storage::Hint;

pub enum Exception {
    UtxoIndexNotEnabled { url: Option<String> },
}

pub struct Wallet {
    interop: Interop,
    wallet: Arc<runtime::Wallet>,
    channel: interop::Channel<Events>,
    section: TypeId,
    sections: HashMap<TypeId, Rc<RefCell<dyn SectionT>>>,
    #[allow(dead_code)]
    settings: Settings,

    is_synced: Option<bool>,
    sync_state: Option<SyncState>,
    server_version: Option<String>,
    url: Option<String>,
    network_id: Option<NetworkId>,
    current_daa_score: Option<u64>,
    hint: Option<Hint>,
    discard_hint: bool,
    exception: Option<Exception>,

    pub wallet_list: Vec<WalletDescriptor>,
    pub account_list: Vec<Arc<dyn runtime::Account>>,
    pub selected_account: Option<Arc<dyn runtime::Account>>,
}

impl Wallet {
    /// Called once before the first frame.
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        interop: crate::interop::Interop,
        settings: Settings,
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        let mut sections = HashMap::<TypeId, Rc<RefCell<dyn SectionT>>>::new();
        sections.insert_typeid(Rc::new(RefCell::new(section::Accounts::new(
            interop.clone(),
        ))));
        sections.insert_typeid(Rc::new(RefCell::new(section::Deposit::new(
            interop.clone(),
        ))));
        sections.insert_typeid(Rc::new(RefCell::new(section::Overview::new(
            interop.clone(),
        ))));
        sections.insert_typeid(Rc::new(RefCell::new(section::Request::new(
            interop.clone(),
        ))));
        sections.insert_typeid(Rc::new(RefCell::new(section::Send::new(interop.clone()))));
        sections.insert_typeid(Rc::new(RefCell::new(section::Settings::new(
            interop.clone(),
        ))));
        sections.insert_typeid(Rc::new(RefCell::new(section::Transactions::new(
            interop.clone(),
        ))));
        sections.insert_typeid(Rc::new(RefCell::new(section::Open::new(interop.clone()))));

        let channel = interop.application_events().clone();
        let wallet = interop.wallet().clone();

        let this = Self {
            interop,
            wallet,
            channel,
            section: TypeId::of::<section::Open>(),
            sections,
            settings,

            wallet_list: Vec::new(),
            account_list: Vec::new(),
            selected_account: None,

            sync_state: None,
            is_synced: None,
            server_version: None,
            url: None,
            network_id: None,
            hint: None,
            discard_hint: false,
            current_daa_score: None,
            exception: None,
        };

        this.wallet_list();

        this
    }

    pub fn select<T>(&mut self)
    where
        T: 'static,
    {
        self.section = TypeId::of::<T>();
    }

    pub fn sender(&self) -> interop::channel::Sender<Events> {
        self.channel.sender.clone()
    }

    pub fn wallet(&self) -> &Arc<runtime::Wallet> {
        &self.wallet
    }

    pub fn rpc_api(&self) -> Arc<DynRpcApi> {
        self.wallet().rpc_api()
    }

    pub fn rpc_client(&self) -> Option<Arc<KaspaRpcClient>> {
        self.rpc_api().clone().downcast_arc::<KaspaRpcClient>().ok()
    }

    // pub fn url(&self) -> String {
    //     self.rpc_client().url().to_string()
    // }

    pub fn get<T>(&self) -> Ref<'_, T>
    where
        T: SectionT + 'static,
    {
        let cell = self.sections.get(&TypeId::of::<T>()).unwrap();
        Ref::map(cell.borrow(), |r| {
            (r).as_any()
                .downcast_ref::<T>()
                .expect("unable to downcast section")
        })
    }

    pub fn get_mut<T>(&mut self) -> RefMut<'_, T>
    where
        T: SectionT + 'static,
    {
        let cell = self.sections.get_mut(&TypeId::of::<T>()).unwrap();
        RefMut::map(cell.borrow_mut(), |r| {
            (r).as_any_mut()
                .downcast_mut::<T>()
                .expect("unable to downcast_mut section")
        })
    }
}

impl eframe::App for Wallet {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // println!("update...");
        for event in self.channel.iter() {
            println!("processing wallet event..");
            if let Err(err) = self.handle_events(event.clone(), ctx, frame) {
                log_error!("error processing wallet interop event: {}", err);
            }
        }

        // let section = self.sections.get(&TypeId::of::<section::Open>()).unwrap().clone();
        // section.borrow_mut().render(self, ctx, frame, ui);
        // return;


        let mut style = (*ctx.style()).clone();
        // println!("style: {:?}", style.text_styles);
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(18.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(18.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(18.0, egui::FontFamily::Proportional),
        );

        // if crate::prompt::prompt().render(ctx) {
        //     return;
        // }

        // if let Some(wizard) = crate::stages::stages() {
        //     if wizard.render_with_context(ctx) {
        //         return;
        //     }
        // }

        // let rect = ctx.screen_rect();
        let size = ctx.screen_rect().size();

        egui::TopBottomPanel::top("top_panel").show(ctx, |_ui| {
            // The top panel is often a good place for a menu bar:
            #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
            egui::menu::bar(_ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            self.render_status(ui);
            egui::warn_if_debug_build(ui);
        });
        /*
        if size.x > 600. {
            egui::SidePanel::left("left_panel").show(&ctx, |ui| {

                if ui.add(egui::Button::new("Overview")).clicked() {
                    // return Stage::Next;
                }
                if ui.add(egui::Button::new("Transactions")).clicked() {
                    // return Stage::Next;
                }

                // let section = self.sections.get(&self.section).unwrap().clone();
                // section.borrow_mut().render(self, ctx, frame, ui);
            });
        }
        */

        egui::CentralPanel::default()
        
        .show(ctx, |ui| {
            ui.style_mut().text_styles = style.text_styles;

            if !self.wallet().is_open() {
                let section = self.sections.get(&TypeId::of::<section::Open>()).unwrap().clone();
                section.borrow_mut().render(self, ctx, frame, ui);
            }
            else
            if size.x > 500. {

                    ui.columns(2, |uis| {
    
                        let section = self.sections.get(&TypeId::of::<section::Overview>()).unwrap().clone();
                        section.borrow_mut().render(self, ctx, frame, &mut uis[0]);
                        let section = self.sections.get(&self.section).unwrap().clone();
                        section.borrow_mut().render(self, ctx, frame, &mut uis[1]);
    
                    });
            } else {
            
                let section = self.sections.get(&self.section).unwrap().clone();
                section.borrow_mut().render(self, ctx, frame, ui);
            
            }

        });

        

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     // ui.style_mut().text_styles = style.text_styles;
        //     let section = self.sections.get(&self.section).unwrap().clone();
        //     section.borrow_mut().render(self, ctx, frame, ui);
        // });

        

/*
        egui::Window::new("main")
        .resize(|r|{
            // r.resizable(false)
            // r.fixed_size(rect.size())
            r.fixed_size(ctx.screen_rect().size())
        })
        // .interactable(false)
        .resizable(false)
        .movable(false)
        .title_bar(false)
        .frame(egui::Frame::none())
        .show(ctx, |ui| {


            egui::TopBottomPanel::top("top_panel").show_inside(ui, |_ui| {
                // The top panel is often a good place for a menu bar:
                #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                egui::menu::bar(_ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            frame.close();
                        }
                    });
                });
            });

            egui::TopBottomPanel::bottom("bottom_panel").show_inside(ui, |ui| {
                self.render_status(ui);
                egui::warn_if_debug_build(ui);
            });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.style_mut().text_styles = style.text_styles;

                let section = self.sections.get(&self.section).unwrap().clone();
                section.borrow_mut().render(self, ctx, frame, ui);
            });
        });
*/

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }
}

impl Wallet {
    fn render_status(&self, ui: &mut egui::Ui) {
        if !self.wallet().is_connected() {
            ui.label("Not Connected");
        } else {
            ui.horizontal(|ui| {
                if self.wallet().is_synced() {
                    self.render_connected_state(ui);
                } else if let Some(status) = self.sync_state.as_ref().map(SyncStatus::try_from) {
                    if status.synced {
                        self.render_connected_state(ui);
                        ui.separator();
                        ui.label("Ready...");
                    } else {
                        ui.vertical(|ui| {
                            status.progress_bar().map(|bar| ui.add(bar));
                            ui.horizontal(|ui| {
                                self.render_connected_state(ui);
                                status.render_text_state(ui);
                                // - TODO - NOT INFO ETC..
                            });
                        });
                    }
                } else {
                    // ui.label("Connected");
                    self.render_connected_state(ui);
                    ui.separator();
                    ui.label("Syncing...");
                }
            });
        }
    }

    fn render_connected_state(&self, ui: &mut egui::Ui) {
        // ui.label(format!("Connected to {}", self.rpc_client().url()));
        ui.label("CONNECTED".to_string());
    }

    pub fn handle_events(
        &mut self,
        event: Events,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) -> Result<()> {
        match event {
            Events::Exit => {
                println!("Exit...");
                cfg_if! {
                    if #[cfg(not(target_arch = "wasm32"))] {
                        _frame.close();
                    }
                }
            }
            Events::Error(_error) => {}
            Events::WalletList { wallet_list } => {
                println!("getting wallet list!, {:?}", wallet_list);
                self.wallet_list = (*wallet_list).clone();
            }
            Events::AccountList { account_list } => {
                self.account_list = (*account_list).clone();
            }
            Events::Close { .. } => {}
            // Events::Send { .. } => { },
            // Events::Deposit { .. } => { },

            // Events::TryUnlock(_secret) => {
            //     let mut unlock = wallet.get_mut::<section::Unlock>();
            //     unlock.message = Some("Error unlocking wallet...".to_string());
            //     unlock.lock();
            // },
            Events::UnlockSuccess => {
                self.select::<section::Overview>();
            }
            Events::UnlockFailure { .. } => {}
            Events::Wallet { event } => {
                match *event {
                    CoreWallet::UtxoProcStart => {

                        // println!("UtxoProcStart...");
                        // self.wallet_list();

                    }
                    CoreWallet::UtxoProcStop => {}
                    CoreWallet::UtxoProcError { message: _ } => {
                        // terrorln!(this,"{err}");
                    }
                    #[allow(unused_variables)]
                    CoreWallet::Connect { url, network_id } => {
                        // log_info!("Connected to {url}");
                        self.url = url;
                        self.network_id = Some(network_id);
                    }
                    #[allow(unused_variables)]
                    CoreWallet::Disconnect {
                        url: _,
                        network_id: _,
                    } => {
                        self.sync_state = None;
                        self.is_synced = None;
                        self.server_version = None;
                        self.url = None;
                        self.network_id = None;
                        self.current_daa_score = None;
                    }
                    CoreWallet::UtxoIndexNotEnabled { url } => {
                        self.exception = Some(Exception::UtxoIndexNotEnabled { url });
                    }
                    CoreWallet::SyncState { sync_state } => {
                        self.sync_state = Some(sync_state);
                    }
                    CoreWallet::ServerStatus {
                        is_synced,
                        server_version,
                        url,
                        network_id,
                    } => {
                        self.is_synced = Some(is_synced);
                        self.server_version = Some(server_version);
                        self.url = url;
                        self.network_id = Some(network_id);
                    }
                    CoreWallet::WalletHint { hint } => {
                        self.hint = hint;
                        self.discard_hint = false;
                    }
                    CoreWallet::WalletOpen | CoreWallet::WalletReload => {
                        self.account_list();
                    }
                    CoreWallet::WalletError { message: _ } => {}
                    CoreWallet::WalletClose => {
                        self.hint = None;
                    }
                    CoreWallet::AccountSelection { id: _ } => {
                        self.selected_account = self.wallet().account().ok();
                    }
                    CoreWallet::DAAScoreChange { current_daa_score } => {
                        self.current_daa_score = Some(current_daa_score);
                    }
                    CoreWallet::Reorg { record: _ } => {}
                    CoreWallet::External { record: _ } => {}
                    CoreWallet::Pending {
                        record: _daa,
                        is_outgoing: _,
                    } => {}
                    CoreWallet::Maturity {
                        record: _,
                        is_outgoing: _,
                    } => {}
                    CoreWallet::Outgoing { record: _ } => {}
                    CoreWallet::Balance {
                        balance: _,
                        id: _,
                        mature_utxo_size: _,
                        pending_utxo_size: _,
                    } => {}
                }
            } // _ => unimplemented!()
        }

        Ok(())
    }

    pub fn wallet_list(&self) {
        let interop = self.interop.clone();
        spawn(async move {
            let wallet_list = Arc::new(interop.wallet().store().wallet_list().await?);
            interop.send(Events::WalletList { wallet_list }).await?;
            Ok(())
        });
    }

    pub fn account_list(&self) {
        let interop = self.interop.clone();
        spawn(async move {
            // let account_map = HashMap::group_from(account_list.into_iter().map(|account| (account.prv_key_data_id().unwrap(), account)));
            let account_list = Arc::new(interop.wallet().activate_all_stored_accounts().await?);
            interop.send(Events::AccountList { account_list }).await?;
            interop
                .wallet()
                .autoselect_default_account_if_single()
                .await
                .ok();
            Ok(())
        });
    }
}
