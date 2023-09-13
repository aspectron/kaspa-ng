use crate::imports::*;
use std::fmt::Display;
pub enum PanelEvents {
    Back,
    Close,
}

pub struct Panel<'panel,Context> {
    pub this : &'panel mut Context,
    // pub icons : &'static Icons,
    // pub ui : &'panel mut egui::Ui,
    caption : Option<String>,
    close : Option<Box<dyn FnOnce(&mut Context) + 'panel>>,
    close_enabled : bool,
    close_active : bool,
    back : Option<Box<dyn FnOnce(&mut Context) + 'panel>>,
    back_enabled : bool,
    back_active : bool,
    header : Option<Box<dyn FnOnce(&mut Context,&mut Ui) + 'panel>>,
    body : Option<Box<dyn FnOnce(&mut Context,&mut Ui) + 'panel>>,
    footer : Option<Box<dyn FnOnce(&mut Context,&mut Ui) + 'panel>>,
}

impl<'panel,Context> Panel<'panel,Context> {

    // const ICONS : &'static Icons = icons();

    // pub fn new(ctx : &'panel mut Context, ui : &'panel mut egui::Ui) -> Self {
    pub fn new(this : &'panel mut Context) -> Self {
        Self {
            this,
            // icons : icons(),
            // ui,
            close: None,
            close_enabled: true,
            close_active: true,
            back : None,
            back_enabled : true,
            back_active : true,
            caption: None,
            header: None,
            body: None,
            footer: None,
        }
    }

    pub fn with_close(mut self, close : impl FnOnce(&mut Context) + 'panel) -> Self {
        self.close = Some(Box::new(close));
        self
    }

    pub fn with_close_enabled(mut self, enabled : bool, close : impl FnOnce(&mut Context) + 'panel) -> Self {
        self.close = Some(Box::new(close));
        self.close_enabled = enabled;
        self
    }

    pub fn with_back(mut self, back : impl FnOnce(&mut Context) + 'panel) -> Self {
        self.back = Some(Box::new(back));
        self
    }

    pub fn with_back_enabled(mut self, enabled : bool, back : impl FnOnce(&mut Context) + 'panel) -> Self {
        self.back = Some(Box::new(back));
        self.back_enabled = enabled;
        self
    }

    pub fn with_caption<S : Display>(mut self, caption : S) -> Self {
        self.caption = Some(caption.to_string());
        self
    }

    pub fn with_header(mut self, header : impl FnOnce(&mut Context, &mut Ui) + 'panel) -> Self {
        self.header = Some(Box::new(header));
        self
    }

    pub fn with_body(mut self, body : impl FnOnce(&mut Context, &mut Ui) + 'panel) -> Self {
        self.body = Some(Box::new(body));
        self
    }

    pub fn with_footer(mut self, footer : impl FnOnce(&mut Context, &mut Ui) + 'panel) -> Self {
        self.footer = Some(Box::new(footer));
        self
    }

    pub fn render(self, ui : &mut Ui) {
            let width = ui.available_width();

            let icon_size = theme().panel_icon_size();

            ui.horizontal(|ui| {
                match self.back {
                    Some(back) if self.back_enabled => {
                        if icons().back.render_with_options(ui, icon_size, self.back_active).clicked() {
                            back(self.this);
                        }
                    },
                    _ => {
                        ui.add_space(icon_size.outer_width());
                    }
                }

                if let Some(caption) = self.caption {
                    ui.add_sized(Vec2::new(width-icon_size.outer_width()*2.,icon_size.outer_height()),Label::new(WidgetText::from(caption).heading()));
                }

                match self.close {
                    Some(close) if self.close_enabled => {
                        if icons().close.render_with_options(ui, icon_size, self.close_active).clicked() {
                            close(self.this);
                        }
                    },
                    _ => {
                        ui.add_space(icon_size.outer_width());
                    }
                }
            });

            ui.add_space(24.);
            
            if let Some(header) = self.header {
                header(self.this, ui);
            }

            ui.add_space(24.);

            // } else {
            //     ui.add_space(icon_size.outer_width());
            // }
            // let height_before
            println!("H1: {}",ui.available_height());
            // println!("used size: {:?}",ui.ctx()..used_size());
            



            // let height = ui.ctx().used_size().y - ui.available_height()
            // let height = ui.available_height()

            // ui.allocate_space(Vec2 { x: width, y: ui.available_height() - 64. });

            egui::ScrollArea::vertical()
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    // ui.
                    // if 
                    // ui.set_height(ui.available_height()-64.);
                    
                    if let Some(body) = self.body {
                        body(self.this, ui);
                    }
                    
                    println!("H2: {}",ui.available_height());
                        // println!("used size in scroll: {:?}",ui.ctx().used_size());

                    // let padding = ui.available_height() - 64.;
                    let padding = ui.available_height() - theme().panel_footer_height;
                    if padding > 0. {
                        ui.add_space(padding);
                    }

                    // ui.

                });

            // let min_height = ui.ctx().used_size().y - 64.;
            // if ui.available_height() < min_height {
            //     ui.set_height(min_height);
            // }
            

            println!("---");
            println!("used size: {:?}",ui.ctx().used_size());
            println!("available height: {}",ui.available_height());
            
            // ui.expand_to_include_y(ui.ctx().used_size().y - 64.);

            // ui.set_height(ui.ctx().used_size().y - 64.);
            println!("used size: {:?}",ui.ctx().used_size());
            println!("available height: {}",ui.available_height());
            println!("---");

            if let Some(footer) = self.footer {
                footer(self.this, ui);
            }
            println!("H3: {}",ui.available_height());

            // println!("used size in after: {:?}",ui.ctx().used_size());
        
    }

}

pub fn panel<'panel, Context>(this : &'panel mut Context) -> Panel<'panel, Context> {
    Panel::new(this)
}
// pub fn panel<'panel, S: Display>(this : &'panel mut Context, caption : S) -> Panel<'panel, Context> {
//     Panel::new(this).with_caption(caption)
// }

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
struct Icons {
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

fn icons() -> &'static Icons {
    static ICONS: OnceLock<Icons> = OnceLock::new();
    ICONS.get_or_init(|| Icons::default())
}
