// use std::sync::Arc;
// use workflow_core::channel::Channel;
use crate::imports::*;


/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Wallet {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    // #[serde(skip)]
    value: f32,
    
    
    // #[serde(skip)]
    wallet: Arc<runtime::Wallet>,

    events : Channel<Events>,

    section : Section,
    // sections : HashMap<Section, Rc<RefCell<dyn Any>>>,
    sections : HashMap<Section, Rc<RefCell<dyn SectionT>>>,
    // accounts : sections::Accounts,
    // deposit : sections::Deposit,
    // overview : sections::Overview,
    // request : sections::Request,

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

impl Wallet {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        let storage = runtime::Wallet::local_store().unwrap_or_else(|e| {
            panic!("Failed to open local store: {}", e);
        });


        let wallet = runtime::Wallet::try_new(storage, None).unwrap_or_else(|e| {
            panic!("Failed to create wallet instance: {}", e);
        });

        let events = Channel::unbounded();

        let sections = {
            // let mut sections = HashMap::<Section,Rc<RefCell<dyn Any>>>::new();
            let mut sections = HashMap::<Section,Rc<RefCell<dyn SectionT>>>::new();
            sections.insert(Section::Accounts, Rc::new(RefCell::new(section::Accounts::new(events.sender.clone()))));
            sections.insert(Section::Deposit, Rc::new(RefCell::new(section::Deposit::new(events.sender.clone()))));
            sections.insert(Section::Overview, Rc::new(RefCell::new(section::Overview::new(events.sender.clone()))));
            sections.insert(Section::Request, Rc::new(RefCell::new(section::Request::new(events.sender.clone()))));
            sections.insert(Section::Send, Rc::new(RefCell::new(section::Send::new(events.sender.clone()))));
            sections.insert(Section::Settings, Rc::new(RefCell::new(section::Settings::new(events.sender.clone()))));
            sections.insert(Section::Transactions, Rc::new(RefCell::new(section::Transactions::new(events.sender.clone()))));
            sections.insert(Section::Unlock, Rc::new(RefCell::new(section::Unlock::new(events.sender.clone()))));
            sections
        };

        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7, 
            wallet : Arc::new(wallet),
            events,
            section: Section::Unlock,
            sections,
        }


    }

    pub fn get<T>(&self, section: Section) -> Ref<'_, T>
    where
        T: SectionT + 'static,
    {
        let cell = self.sections.get(&section).unwrap();
        Ref::map(cell.borrow(), |r| {
            (r).as_any().downcast_ref::<T>().expect("unable to downcast section")
        })
    }

    pub fn get_mut<T>(&mut self, section: Section) -> RefMut<'_, T>
    where
        T: SectionT + 'static,
    {
        let cell = self.sections.get_mut(&section).unwrap();
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

        // self.handle_events();
        while let Ok(event) = self.events.try_recv() {
            event.handle(self).unwrap_or_else(|err|{
                panic!("Failed to handle event `{}` - {err}", event.info());
            })
        }

        let Self { label: _, value: _, .. } = self;

        // - TODO - TRY LISTEN TO WALLET EVENTS AND UPDATE UI

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
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
        let mut style = (*ctx.style()).clone();


        // println!("style: {:?}", style.text_styles);
        // style.text_styles.insert(egui::TextStyle::Button, egui::FontId::new(24.0, egui::FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Body, egui::FontId::new(18.0, egui::FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Button, egui::FontId::new(18.0, egui::FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::new(18.0, egui::FontFamily::Proportional));

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

            egui::warn_if_debug_build(ui);
        });

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
