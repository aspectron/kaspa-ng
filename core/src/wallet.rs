use crate::imports::*;
use crate::interop::Interop;
// use crate::modules::HashMapModuleExtension;
// use crate::modules::HashMapModuleExtension;
use crate::sync::SyncStatus;
use egui_notify::Toasts;
use kaspa_wallet_core::events::Events as CoreWallet;
use kaspa_wallet_core::storage::Hint;

const FORCE_WALLET_OPEN: bool = false;
const ENABLE_DUAL_PANE: bool = false;

pub enum Exception {
    UtxoIndexNotEnabled { url: Option<String> },
}

#[derive(Default)]
pub struct State {
    is_open: bool,
    is_connected: bool,
    is_synced: Option<bool>,
    sync_state: Option<SyncState>,
    server_version: Option<String>,
    url: Option<String>,
    network_id: Option<NetworkId>,
    current_daa_score: Option<u64>,
}

// impl Default for State {
//     fn default() -> Self {
//         State {
//             is_open: None,
//             is_connected: None,
//             is_synced: None,
//             sync_state: None,
//             server_version: None,
//             url: None,
//             network_id: None,
//             current_daa_score: None,
//         }
//     }
// }

impl State {
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    pub fn is_synced(&self) -> bool {
        self.is_synced.unwrap_or(false)
    }

    pub fn sync_state(&self) -> &Option<SyncState> {
        &self.sync_state
    }

    pub fn server_version(&self) -> &Option<String> {
        &self.server_version
    }

    pub fn url(&self) -> &Option<String> {
        &self.url
    }

    pub fn network_id(&self) -> &Option<NetworkId> {
        &self.network_id
    }

    pub fn current_daa_score(&self) -> Option<u64> {
        self.current_daa_score
    }
}

pub struct Wallet {
    interop: Interop,
    wallet: Arc<dyn WalletApi>,
    channel: ApplicationEventsChannel,
    module: TypeId,
    stack: VecDeque<TypeId>,
    // sections: HashMap<TypeId, Rc<RefCell<dyn SectionT>>>,
    modules: HashMap<TypeId, Module>,
    // #[allow(dead_code)]
    pub settings: Settings,

    pub toasts: Toasts,

    pub large_style: egui::Style,
    pub default_style: egui::Style,

    state: State,
    hint: Option<Hint>,
    discard_hint: bool,
    exception: Option<Exception>,

    pub wallet_list: Vec<WalletDescriptor>,
    // pub account_list: Vec<Arc<dyn runtime::Account>>,
    pub account_list: Vec<Account>,
    // pub selected_account: Option<Arc<dyn runtime::Account>>,
    pub selected_account: Option<Account>,
    // pub icons : Icons,
}

impl Wallet {
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        interop: crate::interop::Interop,
        settings: Settings,
    ) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Light);
        cc.egui_ctx.set_fonts(fonts);

        let default_style = (*cc.egui_ctx.style()).clone();
        let mut large_style = (*cc.egui_ctx.style()).clone();
        // println!("style: {:?}", style.text_styles);
        large_style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(22.0, egui::FontFamily::Proportional),
        );
        large_style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(18.0, egui::FontFamily::Proportional),
        );
        large_style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(18.0, egui::FontFamily::Proportional),
        );
        large_style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(18.0, egui::FontFamily::Proportional),
        );

        egui_extras::install_image_loaders(&cc.egui_ctx);

        // cc.egui_ctx.set_style(style);

        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        // let mut sections = HashMap::<TypeId, Rc<RefCell<dyn SectionT>>>::new();
        // let mut modules = HashMap::<TypeId, Module>::new();
        // modules.insert_typeid(modules::Accounts::new(interop.clone()));
        // modules.insert_typeid(modules::Deposit::new(interop.clone()));
        // modules.insert_typeid(modules::Request::new(interop.clone()));
        // modules.insert_typeid(modules::Send::new(interop.clone()));
        // modules.insert_typeid(modules::Settings::new(interop.clone()));
        // modules.insert_typeid(modules::Metrics::new(interop.clone()));
        // modules.insert_typeid(modules::Transactions::new(interop.clone()));
        // modules.insert_typeid(modules::OpenWallet::new(interop.clone()));
        // modules.insert_typeid(modules::CreateWallet::new(interop.clone()));
        // modules.insert_typeid(modules::CreateAccount::new(interop.clone()));
        // modules.insert_typeid(modules::Import::new(interop.clone()));
        // modules.insert_typeid(modules::Export::new(interop.clone()));

        let modules = crate::modules::register_modules(&interop);
        // modules.get_mut_with_typeid::<modules::Settings>().init(&settings);

        // modules.get(&TypeId::of::<modules::Settings>()).unwrap().init(&settings);

        let channel = interop.application_events().clone();
        let wallet = interop.wallet().clone();

        let mut this = Self {
            interop,
            wallet,
            channel,
            module: TypeId::of::<modules::WalletOpen>(),
            modules: modules.clone(),
            stack: VecDeque::new(),
            settings: settings.clone(),
            toasts: Toasts::default(),

            default_style,
            large_style,

            wallet_list: Vec::new(),
            account_list: Vec::new(),
            selected_account: None,

            state: Default::default(),
            // sync_state: None,
            // is_synced: None,
            hint: None,
            discard_hint: false,
            exception: None,
            // icons : Icons::default(),
        };

        modules.values().for_each(|module| {
            module.init(&mut this);
        });

        this.update_wallet_list();

        this
    }

    pub fn select<T>(&mut self)
    where
        T: 'static,
    {
        self.stack.push_back(self.module);

        self.module = TypeId::of::<T>();
        println!("selecting module: {:?}", self.module);
        if self.modules.get(&self.module).is_none() {
            panic!("Unknown module type {:?}", self.module);
        }
    }

    pub fn has_stack(&self) -> bool {
        !self.stack.is_empty()
    }

    pub fn back(&mut self) {
        if let Some(module) = self.stack.pop_back() {
            self.module = module;
        }
    }

    // pub fn select_with_type_id(&mut self, type_id : TypeId)
    // {
    //     self.section = type_id;
    // }

    pub fn sender(&self) -> crate::channel::Sender<Events> {
        self.channel.sender.clone()
    }

    pub fn wallet(&self) -> &Arc<dyn WalletApi> {
        &self.wallet
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    // pub fn wallet(&self) -> &Arc<runtime::Wallet> {
    //     &self.wallet
    // }

    // pub fn rpc_api(&self) -> Arc<DynRpcApi> {
    //     self.wallet().rpc_api()
    // }

    // pub fn rpc_client(&self) -> Option<Arc<KaspaRpcClient>> {
    //     self.rpc_api().clone().downcast_arc::<KaspaRpcClient>().ok()
    // }

    // pub fn network_id(&self) -> Result<NetworkId> {
    //     Ok(self.wallet().network_id()?)
    // }

    pub fn wallet_list(&self) -> &Vec<WalletDescriptor> {
        &self.wallet_list
    }

    pub fn account_list(&self) -> &Vec<Account> {
        &self.account_list
    }

    // pub fn url(&self) -> String {
    //     self.rpc_client().url().to_string()
    // }

    pub fn get<T>(&self) -> Ref<'_, T>
    where
        T: ModuleT + 'static,
    {
        let cell = self.modules.get(&TypeId::of::<T>()).unwrap();
        Ref::map(cell.inner.module.borrow(), |r| {
            (r).as_any()
                .downcast_ref::<T>()
                .expect("unable to downcast section")
        })
    }

    pub fn get_mut<T>(&mut self) -> RefMut<'_, T>
    where
        T: ModuleT + 'static,
    {
        let cell = self.modules.get_mut(&TypeId::of::<T>()).unwrap();
        RefMut::map(cell.inner.module.borrow_mut(), |r| {
            (r).as_any_mut()
                .downcast_mut::<T>()
                .expect("unable to downcast_mut module")
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

        // ctx.set_visuals(self.default_style.clone());
        let current_visuals = ctx.style().visuals.clone(); //.widgets.noninteractive;
        let mut visuals = current_visuals.clone();
        // visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(0, 0, 0));
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0, 0, 0);
        // visuals.bg_fill = egui::Color32::from_rgb(0, 0, 0);
        ctx.set_visuals(visuals);
        self.toasts.show(ctx);
        ctx.set_visuals(current_visuals);

        // let section = self.sections.get(&TypeId::of::<section::Open>()).unwrap().clone();
        // section.borrow_mut().render(self, ctx, frame, ui);
        // return;

        // let mut style = (*ctx.style()).clone();
        // // println!("style: {:?}", style.text_styles);
        // style.text_styles.insert(
        //     egui::TextStyle::Body,
        //     egui::FontId::new(18.0, egui::FontFamily::Proportional),
        // );
        // style.text_styles.insert(
        //     egui::TextStyle::Button,
        //     egui::FontId::new(18.0, egui::FontFamily::Proportional),
        // );
        // style.text_styles.insert(
        //     egui::TextStyle::Monospace,
        //     egui::FontId::new(18.0, egui::FontFamily::Proportional),
        // );

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
            // #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
            egui::menu::bar(_ui, |ui| {
                ui.menu_button("File", |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                    ui.separator();
                    ui.label(" ~ Debug Modules ~");
                    ui.label(" ");

                    let mut modules = self.modules.values().cloned().collect::<Vec<_>>();
                    modules.sort_by(|a, b| a.name().partial_cmp(b.name()).unwrap());
                    modules.into_iter().for_each(|module| {
                        // let SectionInner { name,type_id, .. } = section.inner;
                        if ui.button(module.name()).clicked() {
                            self.module = module.type_id();
                        }
                    });
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().text_styles = self.large_style.text_styles.clone();

            // if false && !self.wallet().is_open() {
            if FORCE_WALLET_OPEN && !self.state.is_open() {
                //self.wallet().is_open() {
                let module = if self.module == TypeId::of::<modules::WalletOpen>()
                    || self.module == TypeId::of::<modules::WalletCreate>()
                {
                    self.module
                } else {
                    TypeId::of::<modules::WalletOpen>()
                };

                // let section = match self.section {
                //      | TypeId::of::<section::Create>() => {
                //         self.section
                //     },
                //     _ => {
                //         self.sections.get(&TypeId::of::<section::Open>()).unwrap().clone()
                //     }
                // };

                // let section = self.sections.get(&section).unwrap().section.clone();
                // section.borrow_mut().render(self, ctx, frame, ui);
                self.modules
                    .get(&module)
                    .unwrap()
                    .clone()
                    .render(self, ctx, frame, ui);
            } else if ENABLE_DUAL_PANE && size.x > 500. {
                ui.columns(2, |uis| {
                    let module = self
                        .modules
                        .get(&TypeId::of::<modules::AccountManager>())
                        .unwrap()
                        .inner
                        .module
                        .clone();
                    module.borrow_mut().render(self, ctx, frame, &mut uis[0]);
                    let module = self.modules.get(&self.module).unwrap().inner.module.clone();
                    module.borrow_mut().render(self, ctx, frame, &mut uis[1]);
                });
            } else {
                let module = self.modules.get(&self.module).unwrap().inner.module.clone();
                module.borrow_mut().render(self, ctx, frame, ui);
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
    fn render_status(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if !self.state().is_connected() {
                ui.label("Not Connected");
            } else {
                ui.horizontal(|ui| {
                    if self.state().is_synced() {
                        self.render_connected_state(ui);
                    } else if let Some(status) =
                        self.state().sync_state.as_ref().map(SyncStatus::try_from)
                    {
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
            ui.add_space(ui.available_width() - 12.);
            // ui.separator();
            if icons()
                .sliders
                .render_with_options(ui, &IconSize::new(Vec2::splat(16.)), true)
                .clicked()
            {
                self.select::<modules::Settings>();
            }
            // ui.label(egui::RichText::new(egui_phosphor::light::GEAR_FINE).size(16.0));
        });
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
                self.wallet_list.sort();
                // self.wallet_list.sort_by(|a, b| {
                //     // a.title.partial_cmp(&b.title).unwrap()
                //     a.filename.partial_cmp(&b.filename).unwrap()
                // });
            }
            Events::AccountList { account_list } => {
                self.account_list = (*account_list).clone();
            }
            Events::Notify { notification } => {
                notification.render(&mut self.toasts);
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
                // self.select::<section::Account>();
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
                        self.state.url = url;
                        self.state.network_id = Some(network_id);
                    }
                    #[allow(unused_variables)]
                    CoreWallet::Disconnect {
                        url: _,
                        network_id: _,
                    } => {
                        self.state.sync_state = None;
                        self.state.is_synced = None;
                        self.state.server_version = None;
                        self.state.url = None;
                        self.state.network_id = None;
                        self.state.current_daa_score = None;
                    }
                    CoreWallet::UtxoIndexNotEnabled { url } => {
                        self.exception = Some(Exception::UtxoIndexNotEnabled { url });
                    }
                    CoreWallet::SyncState { sync_state } => {
                        self.state.sync_state = Some(sync_state);
                    }
                    CoreWallet::ServerStatus {
                        is_synced,
                        server_version,
                        url,
                        network_id,
                    } => {
                        self.state.is_synced = Some(is_synced);
                        self.state.server_version = Some(server_version);
                        self.state.url = url;
                        self.state.network_id = Some(network_id);
                    }
                    CoreWallet::WalletHint { hint } => {
                        self.hint = hint;
                        self.discard_hint = false;
                    }
                    CoreWallet::WalletOpen | CoreWallet::WalletReload => {
                        self.state.is_open = true;
                        self.update_account_list();
                    }
                    CoreWallet::WalletError { message: _ } => {}
                    CoreWallet::WalletClose => {
                        self.hint = None;
                        self.state.is_open = false;
                    }
                    CoreWallet::AccountSelection { id: _ } => {
                        // self.selected_account = self.wallet().account().ok();
                    }
                    CoreWallet::DAAScoreChange { current_daa_score } => {
                        self.state.current_daa_score = Some(current_daa_score);
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

    pub fn update_wallet_list(&self) {
        let interop = self.interop.clone();
        spawn(async move {
            let wallet_list = interop.wallet().wallet_enumerate().await?;
            interop
                .send(Events::WalletList {
                    wallet_list: Arc::new(wallet_list),
                })
                .await?;
            Ok(())
        });
    }

    pub fn update_account_list(&self) {
        let interop = self.interop.clone();
        spawn(async move {
            // let account_map = HashMap::group_from(account_list.into_iter().map(|account| (account.prv_key_data_id().unwrap(), account)));
            let account_list = interop.wallet().account_enumerate().await?; //activate_all_stored_accounts().await?;
            let account_list = account_list
                .into_iter()
                .map(Account::from)
                .collect::<Vec<_>>();
            interop
                .send(Events::AccountList {
                    account_list: Arc::new(account_list),
                })
                .await?;
            // interop
            //     .wallet()
            //     .autoselect_default_account_if_single()
            //     .await
            //     .ok();
            Ok(())
        });
    }
}
