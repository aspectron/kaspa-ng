use crate::frame::window_frame;
use crate::imports::*;
use crate::market::*;
use crate::mobile::MobileMenu;
use egui::load::Bytes;
use egui_notify::Toasts;
use kaspa_wallet_core::api::TransactionsDataGetResponse;
use kaspa_wallet_core::events::Events as CoreWallet;
use kaspa_wallet_core::storage::{Binding, Hint, PrvKeyDataInfo};
use std::borrow::Cow;
#[allow(unused_imports)]
use workflow_i18n::*;
use workflow_wasm::callback::CallbackMap;

pub enum Exception {
    UtxoIndexNotEnabled { url: Option<String> },
}

pub struct Core {
    is_shutdown_pending: bool,
    settings_storage_requested: bool,
    last_settings_storage_request: Instant,

    runtime: Runtime,
    wallet: Arc<dyn WalletApi>,
    application_events_channel: ApplicationEventsChannel,
    deactivation: Option<Module>,
    module: Module,
    modules: HashMap<TypeId, Module>,
    pub stack: VecDeque<Module>,
    pub settings: Settings,
    pub toasts: Toasts,
    pub mobile_style: egui::Style,
    pub default_style: egui::Style,

    state: State,
    hint: Option<Hint>,
    discard_hint: bool,
    exception: Option<Exception>,
    screenshot: Option<Arc<ColorImage>>,

    pub wallet_descriptor: Option<WalletDescriptor>,
    pub wallet_list: Vec<WalletDescriptor>,
    pub prv_key_data_map: Option<HashMap<PrvKeyDataId, Arc<PrvKeyDataInfo>>>,
    pub account_collection: Option<AccountCollection>,
    pub release: Option<Release>,

    pub device: Device,
    pub market: Option<Market>,
    pub debug: bool,
    pub window_frame: bool,
    callback_map: CallbackMap,
    pub network_pressure: NetworkPressure,
    notifications: Notifications,
    pub storage: Storage,
}

impl Core {
    /// Core initialization
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        runtime: crate::runtime::Runtime,
        #[allow(unused_mut)] mut settings: Settings,
        window_frame: bool,
    ) -> Self {
        crate::fonts::init_fonts(cc);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut default_style = (*cc.egui_ctx.style()).clone();

        default_style.text_styles.insert(
            TextStyle::Name("CompositeButtonSubtext".into()),
            FontId {
                size: 10.0,
                family: FontFamily::Proportional,
            },
        );

        let mut mobile_style = (*cc.egui_ctx.style()).clone();

        mobile_style.text_styles.insert(
            TextStyle::Name("CompositeButtonSubtext".into()),
            FontId {
                size: 12.0,
                family: FontFamily::Proportional,
            },
        );

        // println!("style: {:?}", style.text_styles);
        mobile_style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(22.0, egui::FontFamily::Proportional),
        );
        mobile_style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(18.0, egui::FontFamily::Proportional),
        );
        mobile_style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(18.0, egui::FontFamily::Proportional),
        );
        mobile_style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(18.0, egui::FontFamily::Monospace),
            // egui::FontId::new(18.0, egui::FontFamily::Proportional),
        );

        apply_theme_by_name(
            &cc.egui_ctx,
            settings.user_interface.theme_color.as_str(),
            settings.user_interface.theme_style.as_str(),
        );

        // cc.egui_ctx.set_style(style);

        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        let modules: HashMap<TypeId, Module> = {
            cfg_if! {
                if #[cfg(feature = "lean")] {
                    crate::modules::register_generic_modules(&runtime).into_iter().collect()

                } else if #[cfg(target_arch = "wasm32")] {
                    crate::modules::register_generic_modules(&runtime).into_iter().chain(
                        crate::modules::register_advanced_modules(&runtime)
                    ).collect()
                } else {
                    crate::modules::register_generic_modules(&runtime).into_iter().chain(
                        crate::modules::register_advanced_modules(&runtime)
                    ).chain(
                        crate::modules::register_native_modules(&runtime)
                    ).collect()
                }
            }
        };

        let device = Device::new(window_frame);

        #[allow(unused_mut)]
        let mut module_typeid = if workflow_core::runtime::is_chrome_extension() {
            TypeId::of::<modules::WalletOpen>()
        } else {
            TypeId::of::<modules::Overview>()
        };

        #[cfg(not(target_arch = "wasm32"))]
        if settings.version != env!("CARGO_PKG_VERSION") {
            settings.version = env!("CARGO_PKG_VERSION").to_string();
            settings.store_sync().unwrap();

            module_typeid = TypeId::of::<modules::Changelog>();
            // module = modules
            //     .get(&TypeId::of::<modules::Changelog>())
            //     .unwrap()
            //     .clone();
        }

        let module = modules.get(&module_typeid).unwrap().clone();
        // let mut module = modules
        //     .get(&TypeId::of::<modules::Overview>())
        //     .unwrap()
        //     .clone();

        let application_events_channel = runtime.application_events().clone();
        let wallet = runtime.wallet().clone();

        let storage = Storage::default();
        #[cfg(not(target_arch = "wasm32"))]
        if settings.node.kaspad_daemon_storage_folder_enable {
            storage.track_storage_root(Some(settings.node.kaspad_daemon_storage_folder.as_str()));
        }

        let mut this = Self {
            runtime,
            is_shutdown_pending: false,
            settings_storage_requested: false,
            last_settings_storage_request: Instant::now(),

            wallet,
            application_events_channel,
            deactivation: None,
            module,
            modules: modules.clone(),
            stack: VecDeque::new(),
            settings: settings.clone(),
            toasts: Toasts::default(),
            default_style,
            mobile_style,

            wallet_descriptor: None,
            wallet_list: Vec::new(),
            prv_key_data_map: None,
            account_collection: None,
            state: Default::default(),
            hint: None,
            discard_hint: false,
            exception: None,
            screenshot: None,

            release: None,

            device,
            market: None,
            debug: false,
            window_frame,
            callback_map: CallbackMap::default(),
            network_pressure: NetworkPressure::default(),
            notifications: Notifications::default(),
            storage,
            // daemon_storage_root: Mutex::new(daemon_storage_root),
        };

        modules.values().for_each(|module| {
            module.init(&mut this);
        });

        load_public_servers();

        this.wallet_update_list();

        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                this.register_visibility_handler();
            } else {
                let storage = this.storage.clone();
                spawn(async move {
                    loop {
                        storage.update(None);
                        task::sleep(Duration::from_secs(60)).await;
                    }
                });
            }
        }

        this
    }

    pub fn select<T>(&mut self)
    where
        T: 'static,
    {
        self.select_with_type_id(TypeId::of::<T>());
    }

    pub fn select_with_type_id(&mut self, type_id: TypeId) {
        let module = self.modules.get(&type_id).expect("Unknown module");

        if self.module.type_id() != module.type_id() {
            let next = module.clone();
            self.stack.push_back(self.module.clone());
            self.deactivation = Some(self.module.clone());
            self.module = next.clone();
            next.activate(self);

            #[cfg(not(target_arch = "wasm32"))]
            {
                let type_id = self.module.type_id();
                crate::runtime::services::kaspa::update_logs_flag()
                    .store(type_id == TypeId::of::<modules::Logs>(), Ordering::Relaxed);
            }
        }
    }

    pub fn balance_padding(&self) -> bool {
        self.settings.user_interface.balance_padding
    }

    pub fn has_stack(&self) -> bool {
        !self.stack.is_empty()
    }

    pub fn back(&mut self) {
        if self.state().is_open() {
            if let Some(module) = self.stack.pop_back() {
                self.module = module;
            }
        } else {
            while let Some(module) = self.stack.pop_back() {
                if !module.secure() {
                    self.module = module;
                    return;
                }
            }
        }
    }

    pub fn purge_secure_stack(&mut self) {
        self.stack.retain(|module| !module.secure());
    }

    pub fn sender(&self) -> crate::runtime::channel::Sender<Events> {
        self.application_events_channel.sender.clone()
    }

    pub fn store_settings(&self) {
        self.application_events_channel
            .sender
            .try_send(Events::StoreSettings)
            .unwrap();
    }

    pub fn network(&self) -> Network {
        self.settings.node.network
    }

    pub fn wallet(&self) -> Arc<dyn WalletApi> {
        self.wallet.clone()
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn wallet_list(&self) -> &Vec<WalletDescriptor> {
        &self.wallet_list
    }

    pub fn account_collection(&self) -> &Option<AccountCollection> {
        &self.account_collection
    }

    pub fn prv_key_data_map(&self) -> &Option<HashMap<PrvKeyDataId, Arc<PrvKeyDataInfo>>> {
        &self.prv_key_data_map
    }

    pub fn modules(&self) -> &HashMap<TypeId, Module> {
        &self.modules
    }

    pub fn metrics(&self) -> &Option<Box<MetricsSnapshot>> {
        &self.state.node_metrics
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn device_mut(&mut self) -> &mut Device {
        &mut self.device
    }

    pub fn notifications(&mut self) -> &mut Notifications {
        &mut self.notifications
    }

    pub fn module(&self) -> &Module {
        &self.module
    }

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

    pub fn change_current_network(&mut self, network: Network) {
        if self.settings.node.network != network {
            self.settings.node.network = network;
            self.get_mut::<modules::Settings>()
                .change_current_network(network);
            self.store_settings();
            self.runtime
                .kaspa_service()
                .update_services(&self.settings.node, None);
        }
    }
}

impl eframe::App for Core {
    #[cfg(not(target_arch = "wasm32"))]
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.is_shutdown_pending = true;
        crate::runtime::halt();
        println!("{}", i18n("bye!"));
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        for event in self.application_events_channel.iter() {
            if let Err(err) = self.handle_events(event.clone(), ctx, frame) {
                log_error!("error processing wallet runtime event: {}", err);
            }
        }

        if self.is_shutdown_pending {
            return;
        }

        if self.settings_storage_requested
            && self.last_settings_storage_request.elapsed() > Duration::from_secs(5)
        {
            self.settings_storage_requested = false;
            self.settings.store_sync().unwrap();
        }

        ctx.input(|input| {
            input.events.iter().for_each(|event| {
                if let Event::Key {
                    key,
                    pressed,
                    modifiers,
                    repeat,
                } = event
                {
                    self.handle_keyboard_events(*key, *pressed, modifiers, *repeat);
                }
            });

            for event in &input.raw.events {
                if let Event::Screenshot { image, .. } = event {
                    self.screenshot = Some(image.clone());
                }
            }
        });

        // - TODO - TOAST BACKGROUND
        // ---
        let current_visuals = ctx.style().visuals.clone(); //.widgets.noninteractive;
        let mut visuals = current_visuals.clone();
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0, 0, 0);
        ctx.set_visuals(visuals);
        self.toasts.show(ctx);
        ctx.set_visuals(current_visuals);
        // ---

        self.device_mut().set_screen_size(&ctx.screen_rect());

        self.render_frame(ctx, frame);

        if let Some(module) = self.deactivation.take() {
            module.deactivate(self);
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(screenshot) = self.screenshot.clone() {
            self.handle_screenshot(ctx, screenshot);
        }
    }
}

impl Core {
    fn render_frame(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        window_frame(self.window_frame, ctx, "Kaspa NG", |ui| {
            if !self.settings.initialized {
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    self.modules
                        .get(&TypeId::of::<modules::Welcome>())
                        .unwrap()
                        .clone()
                        .render(self, ctx, frame, ui);
                });

                return;
            }

            // delegate rendering to the adaptor, if any
            // return if adaptor consumes the rendering phase
            if let Some(adaptor) = runtime().adaptor() {
                if adaptor.render(self, ui) {
                    return;
                }
            }

            if !self.module.modal() && !self.device.mobile() {
                egui::TopBottomPanel::top("top_panel").show_inside(ui, |ui| {
                    Menu::new(self).render(ui);
                    // self.render_menu(ui, frame);
                });
            }

            if self.device.orientation() == Orientation::Portrait {
                CentralPanel::default()
                    .frame(Frame::default().fill(ctx.style().visuals.panel_fill))
                    .show_inside(ui, |ui| {
                        egui::TopBottomPanel::bottom("portrait_bottom_panel").show_inside(
                            ui,
                            |ui| {
                                Status::new(self).render(ui);
                            },
                        );

                        if self.device.mobile() {
                            egui::TopBottomPanel::bottom("mobile_menu_panel").show_inside(
                                ui,
                                |ui| {
                                    MobileMenu::new(self).render(ui);
                                },
                            );
                        }

                        egui::CentralPanel::default().show_inside(ui, |ui| {
                            self.module.clone().render(self, ctx, frame, ui);
                        });
                    });
            } else if self.device.single_pane() {
                if !self.device.mobile() {
                    egui::TopBottomPanel::bottom("bottom_panel").show_inside(ui, |ui| {
                        Status::new(self).render(ui);
                        egui::warn_if_debug_build(ui);
                    });
                }

                let device_width = 390.0;
                let margin_width = (ctx.screen_rect().width() - device_width) * 0.5;

                SidePanel::right("portrait_right")
                    .exact_width(margin_width)
                    .resizable(false)
                    .show_separator_line(true)
                    .frame(Frame::default().fill(Color32::BLACK))
                    .show_inside(ui, |_ui| {});
                SidePanel::left("portrait_left")
                    .exact_width(margin_width)
                    .resizable(false)
                    .show_separator_line(true)
                    .frame(Frame::default().fill(Color32::BLACK))
                    .show_inside(ui, |_ui| {});

                CentralPanel::default()
                    .frame(Frame::default().fill(ctx.style().visuals.panel_fill))
                    .show_inside(ui, |ui| {
                        ui.set_max_width(device_width);

                        egui::TopBottomPanel::bottom("mobile_bottom_panel").show_inside(ui, |ui| {
                            Status::new(self).render(ui);
                        });

                        if self.device.mobile() {
                            egui::TopBottomPanel::bottom("mobile_menu_panel").show_inside(
                                ui,
                                |ui| {
                                    MobileMenu::new(self).render(ui);
                                },
                            );
                        }

                        egui::CentralPanel::default()
                            .frame(
                                Frame::default()
                                    .inner_margin(0.)
                                    .outer_margin(4.)
                                    .fill(ctx.style().visuals.panel_fill),
                            )
                            .show_inside(ui, |ui| {
                                self.module.clone().render(self, ctx, frame, ui);
                            });
                    });
            } else {
                egui::TopBottomPanel::bottom("bottom_panel")
                    // TODO - review margins
                    .frame(Frame::default().rounding(4.).inner_margin(3.))
                    .show_inside(ui, |ui| {
                        Status::new(self).render(ui);
                        egui::warn_if_debug_build(ui);
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    self.module.clone().render(self, ctx, frame, ui);
                });
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn handle_screenshot(&mut self, ctx: &Context, screenshot: Arc<ColorImage>) {
        match rfd::FileDialog::new().save_file() {
            Some(mut path) => {
                path.set_extension("png");
                let screen_rect = ctx.screen_rect();
                let pixels_per_point = ctx.pixels_per_point();
                let screenshot = screenshot.clone();
                let sender = self.sender();
                std::thread::Builder::new()
                    .name("screenshot".to_string())
                    .spawn(move || {
                        let image = screenshot.region(&screen_rect, Some(pixels_per_point));
                        image::save_buffer(
                            &path,
                            image.as_raw(),
                            image.width() as u32,
                            image.height() as u32,
                            image::ColorType::Rgba8,
                        )
                        .unwrap();

                        sender
                            .try_send(Events::Notify {
                                user_notification: UserNotification::success(format!(
                                    "Capture saved to\n{}",
                                    path.to_string_lossy()
                                ))
                                .as_toast(),
                            })
                            .unwrap()
                    })
                    .expect("Unable to spawn screenshot thread");
                self.screenshot.take();
            }
            None => {
                self.screenshot.take();
            }
        }
    }

    fn _render_splash(&mut self, ui: &mut Ui) {
        let logo_rect = ui.ctx().screen_rect();
        let logo_size = logo_rect.size();
        Image::new(ImageSource::Bytes {
            uri: Cow::Borrowed("bytes://logo.svg"),
            bytes: Bytes::Static(crate::app::KASPA_NG_LOGO_SVG),
        })
        .maintain_aspect_ratio(true)
        // .max_size(logo_size)
        // .fit_to_fraction(vec2(0.9,0.8))
        .fit_to_exact_size(logo_size)
        // .fit_to_exact_size(logo_size)
        // .shrink_to_fit()
        // .bg_fill(Color32::DARK_GRAY)
        .texture_options(TextureOptions::LINEAR)
        // .tint(Color32::from_f32(0.9_f32))
        .paint_at(ui, logo_rect);
    }

    pub fn handle_events(
        &mut self,
        event: Events,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) -> Result<()> {
        // log_info!("--- event: {:?}", event);
        match event {
            // Events::Adaptor { event } => {
            //     if let Some(adaptor) = runtime().adaptor() {
            //         adaptor.handle_event(self, event);
            //     }
            // }
            // Events::WebMessage(msg)=>{
            //     log_info!("Events::WebMessage msg: {msg}");
            // }
            Events::ChangeSection(type_id) => {
                self.select_with_type_id(type_id);
            }
            Events::NetworkChange(network) => {
                self.modules.clone().values().for_each(|module| {
                    module.network_change(self, network);
                });
            }
            Events::UpdateStorage(_options) => {
                #[cfg(not(target_arch = "wasm32"))]
                self.storage
                    .update(Some(_options.with_network(self.settings.node.network)));
            }
            Events::VisibilityChange(state) => match state {
                VisibilityState::Visible => {
                    self.module.clone().show(self);
                }
                VisibilityState::Hidden => {
                    self.module.clone().hide(self);
                }
                _ => {}
            },
            Events::Market(update) => {
                if self.market.is_none() {
                    self.market = Some(Market::default());
                }

                match update {
                    MarketUpdate::Price(price) => {
                        self.market.as_mut().unwrap().price.replace(price);
                    }
                    MarketUpdate::Ohlc(ohlc) => {
                        self.market.as_mut().unwrap().ohlc.replace(ohlc);
                    }
                }
            }
            Events::ThemeChange => {
                if let Some(account_collection) = self.account_collection.as_ref() {
                    account_collection
                        .iter()
                        .for_each(|account| account.update_theme());
                }
            }
            Events::VersionUpdate(release) => {
                self.release = Some(release);
            }
            Events::StoreSettings => {
                self.settings_storage_requested = true;
                self.last_settings_storage_request = Instant::now();
            }
            Events::UpdateLogs => {}
            Events::Metrics { snapshot } => {
                self.state.node_metrics = Some(snapshot);
            }
            Events::MempoolSize { mempool_size } => {
                self.network_pressure
                    .update_mempool_size(mempool_size, &self.settings.node.network);
            }
            Events::Exit => {
                cfg_if! {
                    if #[cfg(not(target_arch = "wasm32"))] {
                        self.is_shutdown_pending = true;
                        _ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                }
            }
            Events::Error(error) => {
                runtime().notify(UserNotification::error(error.as_str()));
            }
            Events::WalletList { wallet_list } => {
                self.wallet_list = (*wallet_list).clone();
                self.wallet_list.sort();
            }
            Events::WalletUpdate => {
                if let Some(account_collection) = self.account_collection.as_ref() {
                    let mut account_manager = self
                        .modules
                        .get(&TypeId::of::<modules::AccountManager>())
                        .unwrap()
                        .clone();
                    account_manager
                        .get_mut::<modules::AccountManager>()
                        .update(account_collection);
                }
            }
            Events::Notify {
                user_notification: notification,
            } => {
                if notification.is_toast() {
                    notification.toast(&mut self.toasts);
                } else {
                    self.notifications.push(notification);
                }
            }
            Events::Close { .. } => {}
            Events::UnlockSuccess => {}
            Events::UnlockFailure { .. } => {}
            Events::PrvKeyDataInfo {
                prv_key_data_info_map,
            } => {
                self.prv_key_data_map = Some(prv_key_data_info_map);
            }
            Events::Wallet { event } => {
                // println!("event: {:?}", event);
                match *event {
                    CoreWallet::WalletPing => {
                        // log_info!("received wallet ping event...");
                        // crate::runtime::runtime().notify(UserNotification::info("Wallet ping"));
                    }
                    CoreWallet::Metrics {
                        network_id: _,
                        metrics,
                    } => {
                        // log_info!("Kaspa NG - received metrics event {metrics:?}");

                        match metrics {
                            MetricsUpdate::WalletMetrics {
                                mempool_size,
                                node_peers: peers,
                                network_tps: tps,
                            } => {
                                self.sender().try_send(Events::MempoolSize {
                                    mempool_size: mempool_size as usize,
                                })?;

                                self.state.node_peers = Some(peers as usize);
                                self.state.node_mempool_size = Some(mempool_size as usize);
                                self.state.network_tps = Some(tps);
                            }
                        }
                    }
                    CoreWallet::Error { message } => {
                        // runtime().notify(UserNotification::error(message.as_str()));
                        println!("{message}");
                    }
                    CoreWallet::UtxoProcStart => {
                        self.state.error = None;

                        if self.state().is_open() {
                            let wallet = self.wallet().clone();
                            spawn(async move {
                                wallet.wallet_reload(false).await?;
                                Ok(())
                            });
                        }
                    }
                    CoreWallet::UtxoProcStop => {}
                    CoreWallet::UtxoProcError { message } => {
                        runtime().notify(UserNotification::error(message.as_str()));

                        if message.contains("network type") {
                            self.state.error = Some(message);
                        }
                    }
                    #[allow(unused_variables)]
                    CoreWallet::Connect { url, network_id } => {
                        // log_info!("Connected to {url:?} on network {network_id}");
                        self.state.is_connected = true;
                        self.state.url = url;
                        self.state.network_id = Some(network_id);

                        self.modules.clone().values().for_each(|module| {
                            module.connect(self, Network::from(network_id));
                        });
                    }
                    #[allow(unused_variables)]
                    CoreWallet::Disconnect {
                        url: _,
                        network_id: _,
                    } => {
                        self.state.is_connected = false;
                        self.state.sync_state = None;
                        self.state.is_synced = None;
                        self.state.server_version = None;
                        self.state.url = None;
                        self.state.network_id = None;
                        self.state.current_daa_score = None;
                        self.state.error = None;
                        self.state.node_metrics = None;
                        self.state.node_peers = None;
                        self.state.node_mempool_size = None;
                        self.network_pressure.clear();

                        self.modules.clone().values().for_each(|module| {
                            module.disconnect(self);
                        });
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
                    CoreWallet::WalletReload {
                        wallet_descriptor,
                        account_descriptors,
                    } => {
                        self.state.is_open = true;

                        self.wallet_descriptor = wallet_descriptor;
                        let network_id = self
                            .state
                            .network_id
                            .unwrap_or(self.settings.node.network.into());
                        let account_descriptors =
                            account_descriptors.ok_or(Error::WalletOpenAccountDescriptors)?;
                        self.load_accounts(network_id, account_descriptors)?;
                    }
                    CoreWallet::WalletOpen {
                        wallet_descriptor,
                        account_descriptors,
                    } => {
                        self.state.is_open = true;

                        self.wallet_descriptor = wallet_descriptor;
                        let network_id = self
                            .state
                            .network_id
                            .unwrap_or(self.settings.node.network.into());
                        let account_descriptors =
                            account_descriptors.ok_or(Error::WalletOpenAccountDescriptors)?;
                        self.load_accounts(network_id, account_descriptors)?;
                    }
                    CoreWallet::WalletCreate {
                        wallet_descriptor,
                        storage_descriptor: _,
                    } => {
                        self.wallet_list.push(wallet_descriptor.clone());
                        self.wallet_descriptor = Some(wallet_descriptor);
                        self.account_collection = Some(AccountCollection::default());
                        self.state.is_open = true;
                    }
                    CoreWallet::PrvKeyDataCreate { prv_key_data_info } => {
                        if let Some(prv_key_data_map) = self.prv_key_data_map.as_mut() {
                            prv_key_data_map
                                .insert(*prv_key_data_info.id(), Arc::new(prv_key_data_info));
                        } else {
                            let mut prv_key_data_map = HashMap::new();
                            prv_key_data_map
                                .insert(*prv_key_data_info.id(), Arc::new(prv_key_data_info));
                            self.prv_key_data_map = Some(prv_key_data_map);
                        }
                    }
                    CoreWallet::AccountDeactivation { ids: _ } => {}
                    CoreWallet::AccountActivation { ids: _ } => {}
                    CoreWallet::AccountCreate {
                        account_descriptor: _,
                    } => {}
                    CoreWallet::AccountUpdate { account_descriptor } => {
                        let account_id = account_descriptor.account_id();
                        if let Some(account_collection) = self.account_collection.as_ref() {
                            if let Some(account) = account_collection.get(account_id) {
                                account.update(account_descriptor);
                            }
                        }
                    }
                    CoreWallet::WalletError { message: _ } => {}
                    CoreWallet::WalletClose => {
                        self.hint = None;
                        self.state.is_open = false;
                        self.account_collection = None;
                        self.wallet_descriptor = None;
                        self.prv_key_data_map = None;

                        self.modules.clone().into_iter().for_each(|(_, module)| {
                            module.reset(self);
                        });

                        self.purge_secure_stack();
                    }
                    CoreWallet::AccountSelection { id } => {
                        if let Some(account_collection) = self.account_collection.as_ref() {
                            if let Some(id) = id {
                                if let Some(account) = account_collection.get(&id) {
                                    let account = account.clone();
                                    let device = self.device().clone();
                                    let wallet = self.wallet();
                                    // log_info!("--- selecting account: {id:?}");
                                    self.get_mut::<modules::AccountManager>().select(
                                        wallet,
                                        Some(account),
                                        device,
                                        false,
                                    );
                                }
                            }
                        }
                    }
                    CoreWallet::DaaScoreChange { current_daa_score } => {
                        self.state.current_daa_score.replace(current_daa_score);
                    }
                    // Ignore scan notifications
                    CoreWallet::Discovery { record } => match record.binding().clone() {
                        Binding::Account(id) => {
                            self.account_collection
                                .as_ref()
                                .and_then(|account_collection| {
                                    account_collection.get(&id).map(|account| {
                                        account.transactions().replace_or_insert(
                                            Transaction::new_confirmed(Arc::new(record)),
                                        );
                                    })
                                });
                        }
                        Binding::Custom(_) => {
                            log_error!("Error while processing transaction {}: custom bindings are not supported", record.id());
                        }
                    },
                    // Ignore stasis notifications
                    CoreWallet::Stasis { record: _ } => {}
                    // A transaction has been confirmed
                    CoreWallet::Maturity { record } => {
                        if record.is_change() {
                            return Ok(());
                        }

                        match record.binding().clone() {
                            Binding::Account(id) => {
                                self.account_collection
                                    .as_ref()
                                    .and_then(|account_collection| {
                                        account_collection.get(&id).map(|account| {
                                            account.transactions().replace_or_insert(
                                                Transaction::new_confirmed(Arc::new(record)),
                                            );
                                        })
                                    });
                            }
                            Binding::Custom(_) => {
                                log_error!("Error while processing transaction {}: custom bindings are not supported", record.id());
                            }
                        }
                    }
                    CoreWallet::Pending { record } => match record.binding().clone() {
                        Binding::Account(id) => {
                            self.account_collection
                                .as_ref()
                                .and_then(|account_collection| {
                                    account_collection.get(&id).map(|account| {
                                        account.transactions().replace_or_insert(
                                            Transaction::new_processing(Arc::new(record)),
                                        );
                                    })
                                });
                        }
                        Binding::Custom(_) => {
                            log_error!("Error while processing transaction {}: custom bindings are not supported", record.id());
                        }
                    },

                    CoreWallet::Reorg { record } => match record.binding().clone() {
                        Binding::Account(id) => {
                            self.account_collection
                                .as_mut()
                                .and_then(|account_collection| {
                                    account_collection
                                        .get(&id)
                                        .map(|account| account.transactions().remove(record.id()))
                                });
                        }
                        Binding::Custom(_) => {
                            log_error!("Error while processing transaction {}: custom bindings are not supported", record.id());
                        }
                    },

                    CoreWallet::Balance { balance, id } => {
                        if let Some(account_collection) = &self.account_collection {
                            if let Some(account) = account_collection.get(&id.into()) {
                                account.update_balance(balance)?;
                            } else {
                                log_error!("unable to find account {}", id);
                            }
                        } else {
                            log_error!(
                                "received CoreWallet::Balance while account collection is empty"
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn wallet_update_list(&self) {
        let runtime = self.runtime.clone();
        spawn(async move {
            let wallet_list = runtime.wallet().wallet_enumerate().await?;
            runtime
                .send(Events::WalletList {
                    wallet_list: Arc::new(wallet_list),
                })
                .await?;
            Ok(())
        });
    }

    fn load_accounts(
        &mut self,
        network_id: NetworkId,
        account_descriptors: Vec<AccountDescriptor>,
    ) -> Result<()> {
        let network = Network::from(network_id);
        if self.network() != network {
            return Err(Error::InvalidNetwork(network.to_string()));
        }

        let application_events_sender = self.application_events_channel.sender.clone();

        let account_list = account_descriptors
            .into_iter()
            .map(|account_descriptor| Account::from(network, account_descriptor))
            .collect::<Vec<_>>();

        self.account_collection = Some(account_list.clone().into());

        let runtime = self.runtime.clone();
        spawn(async move {
            let prv_key_data_info_map = runtime
                .wallet()
                .prv_key_data_enumerate()
                .await?
                .clone()
                .into_iter()
                .map(|prv_key_data_info| (*prv_key_data_info.id(), prv_key_data_info))
                .collect::<HashMap<_, _>>();

            application_events_sender
                .send(Events::PrvKeyDataInfo {
                    prv_key_data_info_map,
                })
                .await?;

            let account_ids = account_list
                .iter()
                .map(|account| account.id())
                .collect::<Vec<_>>();

            let account_map: HashMap<AccountId, Account> = account_list
                .clone()
                .into_iter()
                .map(|account| (account.id(), account))
                .collect::<HashMap<_, _>>();

            // TODO - finish progressive transaction loading implementation
            let futures = account_ids
                .into_iter()
                .map(|account_id| {
                    runtime
                        .wallet()
                        .transactions_data_get_range(account_id, network_id, 0..16384)
                })
                .collect::<Vec<_>>();

            let transaction_data = join_all(futures)
                .await
                .into_iter()
                .map(|v| v.map_err(Error::from))
                .collect::<Result<Vec<_>>>()?;

            transaction_data.into_iter().for_each(|data| {
                let TransactionsDataGetResponse {
                    account_id,
                    transactions,
                    start: _,
                    total,
                } = data;

                if let Some(account) = account_map.get(&account_id) {
                    if let Err(err) = account.load_transactions(transactions, total) {
                        log_error!("error loading transactions into account {account_id}: {err}");
                    }
                } else {
                    log_error!("unable to find account {}", account_id);
                }
            });

            runtime.wallet().accounts_activate(None).await?;

            application_events_sender.send(Events::WalletUpdate).await?;

            Ok(())
        });

        Ok(())
    }

    pub fn handle_account_creation(
        &mut self,
        account_descriptors: Vec<AccountDescriptor>,
    ) -> Vec<Account> {
        let network = self.network();

        let accounts = account_descriptors
            .into_iter()
            .map(|account_descriptor| Account::from(network, account_descriptor))
            .collect::<Vec<_>>();

        self.account_collection
            .as_mut()
            .expect("account collection")
            .extend_unchecked(accounts.clone());

        if let Some(first) = accounts.first() {
            let device = self.device().clone();
            let wallet = self.wallet();
            self.get_mut::<modules::AccountManager>().select(
                wallet,
                Some(first.clone()),
                device,
                true,
            );
        }

        let account_ids = accounts
            .iter()
            .map(|account| account.id())
            .collect::<Vec<_>>();

        let wallet = self.wallet().clone();
        spawn(async move {
            wallet.accounts_activate(Some(account_ids)).await?;
            Ok(())
        });

        accounts
    }

    fn handle_keyboard_events(
        &mut self,
        key: Key,
        pressed: bool,
        modifiers: &Modifiers,
        _repeat: bool,
    ) {
        if !pressed {
            return;
        }

        if modifiers.ctrl || modifiers.mac_cmd {
            match key {
                Key::O => {
                    self.select::<modules::WalletOpen>();
                }
                Key::N => {
                    self.select::<modules::WalletCreate>();
                }
                Key::M => {
                    self.device_mut().toggle_mobile();
                }
                Key::P => {
                    self.device_mut().toggle_portrait();
                }
                Key::D => {
                    self.debug = !self.debug;
                }
                _ => {}
            }
        }

        if (modifiers.ctrl || modifiers.mac_cmd) && modifiers.shift {
            match key {
                Key::T => {
                    self.select::<modules::Testing>();
                }
                Key::M => {
                    self.device_mut().toggle_mobile();
                }
                Key::P => {
                    self.device_mut().toggle_portrait();
                }
                _ => {}
            }
        }
    }

    pub fn apply_mobile_style(&self, ui: &mut Ui) {
        ui.style_mut().text_styles = self.mobile_style.text_styles.clone();
    }

    pub fn apply_default_style(&self, ui: &mut Ui) {
        ui.style_mut().text_styles = self.default_style.text_styles.clone();
    }

    pub fn register_visibility_handler(&self) {
        use workflow_wasm::callback::*;

        #[cfg(not(feature = "lean"))]
        let block_dag_background_state = self.get::<modules::BlockDag>().background_state();

        let sender = self.sender();
        let callback = callback!(move || {
            let visibility_state = document().visibility_state();

            #[cfg(not(feature = "lean"))]
            match visibility_state {
                VisibilityState::Visible => {
                    let block_dag_monitor_service = crate::runtime::runtime()
                        .block_dag_monitor_service()
                        .clone();
                    if block_dag_monitor_service.is_active() {
                        block_dag_monitor_service.enable(None);
                    }
                }
                VisibilityState::Hidden => {
                    let block_dag_monitor_service = crate::runtime::runtime()
                        .block_dag_monitor_service()
                        .clone();
                    if !block_dag_background_state.load(Ordering::SeqCst)
                        && block_dag_monitor_service.is_active()
                    {
                        block_dag_monitor_service.disable(None);
                    }
                }
                _ => {}
            };
            sender
                .try_send(Events::VisibilityChange(visibility_state))
                .unwrap();
            runtime().egui_ctx().request_repaint();
        });

        document().set_onvisibilitychange(Some(callback.as_ref()));
        self.callback_map.retain(callback).unwrap();
    }
}
