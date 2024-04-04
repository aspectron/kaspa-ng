use crate::{imports::*, modules::account_manager::AccountManagerSection};
use egui_extras::{Size, StripBuilder};
use egui_phosphor::thin::*;

type MobileMenuHandler<'handler> = dyn FnOnce(&mut Core, &mut Ui) + 'handler;

pub struct Handler<'handler> {
    icon: &'handler str,
    text: &'handler str,
    handler: Box<MobileMenuHandler<'handler>>,
}

impl<'handler> Handler<'handler> {
    pub fn new(
        icon: &'handler str,
        text: &'handler str,
        handler: Box<MobileMenuHandler<'handler>>,
    ) -> Self {
        Self {
            icon,
            text,
            handler,
        }
    }
}

pub struct MobileMenu<'core> {
    core: &'core mut Core,
}

impl<'core> MobileMenu<'core> {
    pub fn new(core: &'core mut Core) -> Self {
        Self { core }
    }

    pub fn render(&mut self, ui: &mut Ui) {
        if self.core.state().is_open() {
            self.render_open(ui);
        } else {
            self.render_closed(ui);
        }
    }

    pub fn render_closed(&mut self, ui: &mut Ui) {
        let handlers = vec![Handler::new(
            FINGERPRINT,
            "OPEN",
            Box::new(|core, _ui| {
                core.select::<modules::WalletOpen>();
            }),
        )];

        self.render_strip(ui, handlers);
    }

    pub fn render_open(&mut self, ui: &mut Ui) {
        let mut handlers = vec![
            // Handler::new(HOUSE, "HOME", Box::new(|core, _ui| {
            //     core.select::<modules::AccountManager>();
            // })),
            Handler::new(
                WALLET,
                "ACCOUNT",
                Box::new(|core, _ui| {
                    core.get_mut::<modules::AccountManager>()
                        .section(AccountManagerSection::Overview);
                    core.select::<modules::AccountManager>();
                }),
            ),
            Handler::new(
                LIST_BULLETS,
                "TRANSACTIONS",
                Box::new(|core, _ui| {
                    core.get_mut::<modules::AccountManager>()
                        .section(AccountManagerSection::Transactions);
                    core.select::<modules::AccountManager>();
                }),
            ),
            Handler::new(
                SLIDERS,
                "SETTINGS",
                Box::new(|core, _ui| {
                    core.select::<modules::Settings>();
                }),
            ),
        ];

        let account_collection = self
            .core
            .account_collection
            .as_ref()
            .expect("account collection");

        if account_collection.len() > 1 {
            handlers.insert(
                0,
                Handler::new(
                    HOUSE_SIMPLE,
                    "HOME",
                    Box::new(|core, _ui| {
                        let device = core.device().clone();
                        let wallet = core.wallet();
                        core.get_mut::<modules::AccountManager>()
                            .select(wallet, None, device, true);
                        core.select::<modules::AccountManager>();
                    }),
                ),
            );
        }

        self.render_strip(ui, handlers);
    }

    fn render_strip(&mut self, ui: &mut Ui, handlers: Vec<Handler<'_>>) {
        let mut strip_builder = StripBuilder::new(ui).cell_layout(Layout::top_down(Align::Center));

        for _ in handlers.iter() {
            strip_builder = strip_builder.size(Size::remainder());
        }

        strip_builder.horizontal(|mut strip| {
            for handler in handlers.into_iter() {
                let Handler {
                    icon,
                    text,
                    handler,
                } = handler;
                strip.cell(|ui| {
                    ui.vertical_centered(|ui| {
                        if ui
                            .add(Label::new(RichText::new(icon).size(48.)).sense(Sense::click()))
                            .clicked()
                        {
                            handler(self.core, ui);
                        }

                        ui.label(RichText::new(text).size(8.));
                    });
                });
            }
        });
    }
}
