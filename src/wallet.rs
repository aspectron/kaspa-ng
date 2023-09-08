    use kaspa_wallet_core::{DynRpcApi, SyncState};

// use std::sync::Arc;
// use workflow_core::channel::Channel;
use crate::imports::*;
use crate::interop::Interop;
use crate::sync::SyncStatus;
use kaspa_wallet_core::events::Events as CoreWallet;

// pub static mut CTX: Option<egui::Context> = None;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Wallet {
    interop : Interop,
    wallet : Arc<runtime::Wallet>,
    channel : interop::Channel<Events>,
    section : TypeId,
    sections : HashMap<TypeId, Rc<RefCell<dyn SectionT>>>,

    sync_state : Option<SyncState>,
    // ctx: Option<egui::Context>,
}

// impl Default for KaspaWallet {
//     fn default() -> Self {
//         Self {
//             // Example stuff:
//             label: "Hello World!".to_owned(),
//             value: 2.7, 
//         }
//     }
// }

trait HashMapTypeIdExtension<T> {
    // fn insert_typeid<T>(&mut self, value: Rc<RefCell<T>>)
    fn insert_typeid(&mut self, value: Rc<RefCell<T>>)
    where
        T: SectionT + 'static;
}

// impl<V> HashMapTypeIdExtension for HashMap<TypeId,V> 
// impl<T> HashMapTypeIdExtension<T> for HashMap<TypeId,Rc<RefCell<T>>> 
impl<T> HashMapTypeIdExtension<T> for HashMap<TypeId,Rc<RefCell<dyn SectionT>>> 
where T : SectionT
{
    // fn insert_typeid<T>(&mut self, value: Rc<RefCell<T>>)
    fn insert_typeid(&mut self, value: Rc<RefCell<T>>)
    // where
    // K: Eq + Hash + 'static,
        // T: SectionT + 'static,
    {
        self.insert(TypeId::of::<T>(), value);
    }
}

impl Wallet {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>, interop : crate::interop::Interop) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }


        let events = interop.channel().clone(); //Channel::unbounded();

        let mut sections = HashMap::<TypeId,Rc<RefCell<dyn SectionT>>>::new();
        sections.insert_typeid(Rc::new(RefCell::new(section::Accounts::new(interop.clone()))));
        sections.insert_typeid(Rc::new(RefCell::new(section::Deposit::new(interop.clone()))));
        sections.insert_typeid(Rc::new(RefCell::new(section::Overview::new(interop.clone()))));
        sections.insert_typeid(Rc::new(RefCell::new(section::Request::new(interop.clone()))));
        sections.insert_typeid(Rc::new(RefCell::new(section::Send::new(interop.clone()))));
        sections.insert_typeid(Rc::new(RefCell::new(section::Settings::new(interop.clone()))));
        sections.insert_typeid(Rc::new(RefCell::new(section::Transactions::new(interop.clone()))));
        sections.insert_typeid(Rc::new(RefCell::new(section::Unlock::new(interop.clone()))));

        let wallet = interop.wallet_service().wallet().clone();

        Self {
            // Example stuff:
            // label: "Hello World!".to_owned(),
            // value: 2.7, 
            // wallet : interop.wallet(), //Arc::new(wallet),
            interop,
            wallet,
            channel: events,
            section: TypeId::of::<section::Unlock>(),
            sections,
            sync_state : None,
        }


    }

    // pub fn 

    // pub fn select(&mut self, section : Section) -> RefMut<'_, {
    pub fn select<T>(&mut self) 
    where T : 'static
    {
        self.section = TypeId::of::<T>();
    }

    pub fn sender(&self) -> interop::channel::Sender<Events> {
        self.channel.sender.clone()
    }

    pub fn wallet(&self) -> &Arc<runtime::Wallet> {
        &self.wallet
    }

    pub fn rpc(&self) -> &Arc<DynRpcApi> {
        self.wallet().rpc()
    }

    pub fn rpc_client(&self) -> Arc<KaspaRpcClient> {
        self.rpc().clone().downcast_arc::<KaspaRpcClient>().expect("unable to downcast DynRpcApi to KaspaRpcClient")
    }

    // pub fn url(&self) -> String {
    //     self.rpc_client().url().to_string()
    // }

    // pub fn connection_string

    // - TODO - USE TYPEID 
    // if (*r).type_id() == TypeId::of::<T>() {

    pub fn get<T>(&self) -> Ref<'_, T>
    where
        T: SectionT + 'static,
    {
        let cell = self.sections.get(&TypeId::of::<T>()).unwrap();
        Ref::map(cell.borrow(), |r| {
            (r).as_any().downcast_ref::<T>().expect("unable to downcast section")
        })
    }

    pub fn get_mut<T>(&mut self) -> RefMut<'_, T>
    where
        T: SectionT + 'static,
    {
        let cell = self.sections.get_mut(&TypeId::of::<T>()).unwrap();
        RefMut::map(cell.borrow_mut(), |r| {
            (r).as_any_mut().downcast_mut::<T>().expect("unable to downcast_mut section")
        })
    }

    
}

impl eframe::App for Wallet {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) { 

        // unsafe {
        //     if CTX.is_none() {
        //         CTX.replace(ctx.clone());
        //     }
        // }

        // self.handle_events();
        while let Ok(event) = self.channel.try_recv() {
            self.handle_events(event.clone(), ctx, frame).unwrap_or_else(|err|{
                panic!("Failed to handle event `{}` - {err}", event.info());
            })
        }

        // let Self { label: _, value: _, .. } = self;

        // - TODO - TRY LISTEN TO WALLET EVENTS AND UPDATE UI

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui


        let mut style = (*ctx.style()).clone();
        // println!("style: {:?}", style.text_styles);
        // style.text_styles.insert(egui::TextStyle::Button, egui::FontId::new(24.0, egui::FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Body, egui::FontId::new(18.0, egui::FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Button, egui::FontId::new(18.0, egui::FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::new(18.0, egui::FontFamily::Proportional));


        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
            egui::menu::bar(ui, |ui| {
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().text_styles = style.text_styles;

            // The central panel the region left after adding TopPanel's and SidePanel's

            // ui.heading("Kaspa Wallet");
            // ui.hyperlink("https://github.com/emilk/eframe_template");
            // ui.add(egui::github_link_file!(
            //     "https://github.com/emilk/eframe_template/blob/master/",
            //     "Source code."
            // ));

            let section = self.sections.get(&self.section).unwrap().clone();
            section.borrow_mut().render(self, ctx, frame, ui);

        });

        // egui::SidePanel::left("side_panel").show(ctx, |ui| {
        //     ui.heading("Side Panel");

        //     ui.horizontal(|ui| {
        //         ui.label("Write something: ");
        //         ui.text_edit_singleline(label);
        //     });

        //     ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
        //     if ui.button("Increment").clicked() {
        //         *value += 1.0;
        //     }

        //     ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        //         ui.horizontal(|ui| {
        //             ui.spacing_mut().item_spacing.x = 0.0;
        //             ui.label("powered by ");
        //             ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        //             ui.label(" and ");
        //             ui.hyperlink_to(
        //                 "eframe",
        //                 "https://github.com/emilk/egui/tree/master/crates/eframe",
        //             );
        //             ui.label(".");
        //         });
        //     });
        // });


        if true {
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
    fn render_status(&self, ui : &mut egui::Ui) {

        if !self.wallet().is_connected() {
            ui.label("Not Connected");
        } else {
            ui.horizontal(|ui| {
                
                if self.wallet().is_synced() {
                    self.render_connected_state(ui);
                } else {
                    if let Some(status) = self.sync_state.as_ref().map(SyncStatus::try_from) {

                        if status.synced {
                            self.render_connected_state(ui);
                            ui.separator();
                            ui.label("Ready...");
                        } else {

                            ui.vertical(|ui| {
                                status.progress_bar().map(|bar|ui.add(bar));
                                ui.horizontal(|ui|{
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
                }
            });
        }

    }
    
    // fn render_sync_prefix(&self, ui : &mut egui::Ui) {
    //     ui.label("Connected");
    //     ui.separator();
    // }

    fn render_sync_state(&self, ui : &mut egui::Ui) {

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Sync State");
                ui.separator();
            });
            ui.horizontal(|ui| {
                ui.label("Sync State");
                ui.separator();
            });
        });

    }

    fn render_connected_state(&self, ui : &mut egui::Ui) {
        // if true {

            ui.label(format!("Connected to {}", self.rpc_client().url()));
        // }
    }





    pub fn handle_events(&mut self, event : Events, ctx: &egui::Context, frame: &mut eframe::Frame) -> Result<()> {
        match event {
            Events::Exit => {
                frame.close();
                // wallet.exit();
            },
            Events::Error(_error) => {

            }
            // Events::TryUnlock(_secret) => {
            //     let mut unlock = wallet.get_mut::<section::Unlock>();
            //     unlock.message = Some("Error unlocking wallet...".to_string());
            //     unlock.lock();
            // },
            Events::UnlockSuccess => {
                self.select::<section::Overview>();
            },
            Events::UnlockFailure {..} => {

            },
            Events::Wallet(event) => {
                match event {
                    CoreWallet::UtxoProcStart => {},
                    CoreWallet::UtxoProcStop => {},
                    CoreWallet::UtxoProcError(_err) => {
                        // terrorln!(this,"{err}");
                    },
                    #[allow(unused_variables)]
                    CoreWallet::Connect{ url, network_id } => {
                        // log_info!("Connected to {url}");
                    },
                    #[allow(unused_variables)]
                    CoreWallet::Disconnect{ url, network_id } => {
                        // tprintln!(this, "Disconnected from {url}");
                        // this.term().refresh_prompt();
                    },
                    CoreWallet::UtxoIndexNotEnabled => {
                        // tprintln!(this, "Error: Kaspa node UTXO index is not enabled...")
                    },
                    CoreWallet::SyncState(_state) => {
                        // this.sync_state.lock().unwrap().replace(state);
                        // this.term().refresh_prompt();
                    }
                    CoreWallet::ServerStatus {
                        is_synced:_,
                        server_version:_,
                        url:_,
                        ..
                    } => {

                        // tprintln!(this, "Connected to Kaspa node version {server_version} at {url}");

                        // let is_open = this.wallet.is_open();

                        // if !is_synced {
                        //     if is_open {
                        //         terrorln!(this, "Unable to update the wallet state - Kaspa node is currently syncing with the network...");

                        //     } else {
                        //         terrorln!(this, "Kaspa node is currently syncing with the network, please wait for the sync to complete...");
                        //     }
                        // }

                        // this.term().refresh_prompt();

                    },
                    CoreWallet::WalletHint {
                        hint:_
                    } => {

                        // if let Some(hint) = hint {
                        //     tprintln!(this, "\nYour wallet hint is: {hint}\n");
                        // }

                    },
                    CoreWallet::WalletOpen |
                    CoreWallet::WalletReload => {

                        // load all accounts
                        // this.wallet().activate_all_stored_accounts().await.unwrap_or_else(|err|terrorln!(this, "{err}"));

                        // // list all accounts
                        // this.list().await.unwrap_or_else(|err|terrorln!(this, "{err}"));

                        // // load default account if only one account exists
                        // this.wallet().autoselect_default_account_if_single().await.ok();
                        // this.term().refresh_prompt();

                    },
                    CoreWallet::WalletError { message: _ } => {

                    },
                    CoreWallet::WalletClose => {
                        // this.term().refresh_prompt();
                    },
                    CoreWallet::AccountSelection { id: _ } => {

                    }
                    CoreWallet::DAAScoreChange(_daa) => {
                        // if this.is_mutted() && this.flags.get(Track::Daa) {
                        //     tprintln!(this, "{NOTIFY} DAA: {daa}");
                        // }
                    },
                    CoreWallet::Reorg {
                        record:_
                    } => {
                        // if !this.is_mutted() || (this.is_mutted() && this.flags.get(Track::Pending)) {
                        //     let include_utxos = this.flags.get(Track::Utxo);
                        //     let tx = record.format_with_state(&this.wallet,Some("reorg"),include_utxos).await;
                        //     tx.iter().for_each(|line|tprintln!(this,"{NOTIFY} {line}"));
                        // }
                    },
                    CoreWallet::External {
                        record:_
                    } => {
                        // if !this.is_mutted() || (this.is_mutted() && this.flags.get(Track::Tx)) {
                        //     let include_utxos = this.flags.get(Track::Utxo);
                        //     let tx = record.format_with_state(&this.wallet,Some("external"),include_utxos).await;
                        //     tx.iter().for_each(|line|tprintln!(this,"{NOTIFY} {line}"));
                        // }
                    },
                    CoreWallet::Pending {
                        record:_daa, is_outgoing : _
                    } => {
                        // if !this.is_mutted() || (this.is_mutted() && this.flags.get(Track::Pending)) {
                        //     let include_utxos = this.flags.get(Track::Utxo);
                        //     let tx = record.format_with_state(&this.wallet,Some("pending"),include_utxos).await;
                        //     tx.iter().for_each(|line|tprintln!(this,"{NOTIFY} {line}"));
                        // }
                    },
                    CoreWallet::Maturity {
                        record:_, is_outgoing : _
                    } => {
                        // if !this.is_mutted() || (this.is_mutted() && this.flags.get(Track::Tx)) {
                        //     let include_utxos = this.flags.get(Track::Utxo);
                        //     let tx = record.format_with_state(&this.wallet,Some("confirmed"),include_utxos).await;
                        //     tx.iter().for_each(|line|tprintln!(this,"{NOTIFY} {line}"));
                        // }
                    },
                    CoreWallet::Outgoing {
                        record:_
                    } => {
                        // if !this.is_mutted() || (this.is_mutted() && this.flags.get(Track::Tx)) {
                        //     let include_utxos = this.flags.get(Track::Utxo);
                        //     let tx = record.format_with_state(&this.wallet,Some("confirmed"),include_utxos).await;
                        //     tx.iter().for_each(|line|tprintln!(this,"{NOTIFY} {line}"));
                        // }
                    },
                    CoreWallet::Balance {
                        balance:_,
                        id:_,
                        mature_utxo_size:_,
                        pending_utxo_size:_,
                    } => {

                        // if !this.is_mutted() || (this.is_mutted() && this.flags.get(Track::Balance)) {
                        //     let network_id = this.wallet.network_id().expect("missing network type");
                        //     let network_type = NetworkType::from(network_id);
                        //     let balance = BalanceStrings::from((&balance,&network_type, None));
                        //     let id = id.short();

                        //     let pending_utxo_info = if pending_utxo_size > 0 {
                        //         format!("({pending_utxo_size} pending)")
                        //     } else { "".to_string() };
                        //     let utxo_info = style(format!("{} UTXOs {pending_utxo_info}", mature_utxo_size.separated_string())).dim();

                        //     tprintln!(this, "{NOTIFY} {} {id}: {balance}   {utxo_info}",style("balance".pad_to_width(8)).blue());
                        // }

                        // this.term().refresh_prompt();
                    }
                }
            }
            _ => unimplemented!()
        }

        Ok(())        
    }

}

