use crate::imports::*;
use std::fmt::Display;
pub enum PanelEvents {
    Back,
    Close,
}

pub struct Panel<'panel,Context> {
    pub ctx : &'panel mut Context,
    // pub icons : &'static Icons,
    // pub ui : &'panel mut egui::Ui,
    caption : Option<String>,
    close : Option<Box<dyn FnOnce(&mut Context) + 'static>>,
    back : Option<Box<dyn FnOnce(&mut Context) + 'static>>,
    header : Option<Box<dyn FnOnce(&mut Context,&mut Ui) + 'static>>,
    body : Option<Box<dyn FnOnce(&mut Context,&mut Ui) + 'panel>>,
    footer : Option<Box<dyn FnOnce(&mut Context,&mut Ui) + 'static>>,
}

impl<'panel,Context> Panel<'panel,Context> {

    // const ICONS : &'static Icons = icons();

    // pub fn new(ctx : &'panel mut Context, ui : &'panel mut egui::Ui) -> Self {
    pub fn new(ctx : &'panel mut Context) -> Self {
        Self {
            ctx,
            // icons : icons(),
            // ui,
            close: None,
            back : None,
            caption: None,
            header: None,
            body: None,
            footer: None,
        }
    }

    pub fn with_close(mut self, close : impl FnOnce(&mut Context) + 'static) -> Self {
        self.close = Some(Box::new(close));
        self
    }

    pub fn with_back(mut self, back : impl FnOnce(&mut Context) + 'static) -> Self {
        self.back = Some(Box::new(back));
        self
    }

    pub fn with_caption<S : Display>(mut self, caption : S) -> Self {
        self.caption = Some(caption.to_string());
        self
    }

    pub fn with_header(mut self, header : impl FnOnce(&mut Context, &mut Ui) + 'static) -> Self {
        self.header = Some(Box::new(header));
        self
    }

    pub fn with_body(mut self, body : impl FnOnce(&mut Context, &mut Ui) + 'panel) -> Self {
    // pub fn with_body(mut self, body : impl FnOnce(&mut Ui) + 'panel) -> Self {
    // pub fn with_body(mut self, body : &'panel dyn FnOnce(&mut Ui)) -> Self {
        // self.body = Some(body);
        self.body = Some(Box::new(body));
        self
    }

    pub fn with_footer(mut self, footer : impl FnOnce(&mut Context, &mut Ui) + 'static) -> Self {
        self.footer = Some(Box::new(footer));
        self
    }

    pub fn render(self, ui : &mut Ui) {
            let width = ui.available_width();

            let icon_size = theme().panel_icon_size();

            ui.horizontal(|ui| {
                if let Some(back) = self.back {
                    if icons().back.render(ui, icon_size).clicked() {
                        println!("RECEIVED CLICK TO BACK!");
                        // back(&mut self.ctx);
                        back(self.ctx);
                    }
                } else {
                    ui.add_space(icon_size.outer_width());
                }

                if let Some(caption) = self.caption {
                    ui.add_sized(Vec2::new(width-icon_size.outer_width()*2.,icon_size.outer_height()),Label::new(WidgetText::from(caption).heading()));
                }

                if let Some(close) = self.close {
                    if icons().close.render(ui, icon_size).clicked() {
                        close(self.ctx);
                    }
                } else {
                    ui.add_space(icon_size.outer_width());
                }
            });

            
            if let Some(header) = self.header {
                header(self.ctx, ui);
            } else {
                ui.add_space(24.);
            }

            // ui.label("Select a wallet to unlock");
            // ui.label(" ");
            // ui.add_space(32.);

            egui::ScrollArea::vertical()
                // .id_source("wallet-list")
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(ui.available_height()-64.);

                    if let Some(body) = self.body {
                        body(self.ctx, ui);
                        // (*body)(ui);
                    }
                });

            if let Some(footer) = self.footer {
                footer(self.ctx, ui);
            }
        
    }

}

// pub trait PanelExtension<'ui,Context> {
//     fn panel(&'ui mut self, ctx : &mut Context, caption : &'static str) -> Panel<'ui,Context>;
// }

// impl<'ui,Context> PanelExtension<'ui,Context> for Ui {
//     fn panel(&'ui mut self, ctx: &mut Context, caption: &'static str) -> Panel<'ui,Context> {
//         Panel::new(ctx, self).with_caption(caption.to_string())
//     }
// }

macro_rules! phosphor {
    ($symbol:ident) => (
        Icon::new(egui_phosphor::regular::$symbol)
    );
}

// #[derive(Clone)]
pub struct Icons {
    pub back : Icon,
    pub close : Icon,
}

impl Default for Icons {
    fn default() -> Self {
        Self {
            back : phosphor!(ARROW_BEND_UP_LEFT),
            close : phosphor!(X),
        }
    }
}

pub fn icons() -> &'static Icons {
    static ICONS: OnceLock<Icons> = OnceLock::new();
    ICONS.get_or_init(|| Icons::default())
}
