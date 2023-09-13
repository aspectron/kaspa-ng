use crate::imports::*;
pub use egui_phosphor::*;

#[derive(Clone)]
pub enum Kind {
    Phosphor { symbol : &'static str }
}

pub struct IconSize {
    pub inner : Vec2,
    pub outer : Vec2,
}

impl IconSize {
    pub fn new(inner : Vec2) -> Self {
        Self { inner, outer : inner }
    }

    pub fn with_padding(mut self, padding : Vec2) -> Self {
        self.outer.x += padding.x * 2.;
        self.outer.y += padding.y * 2.;
        self
    }

    pub fn new_square(inner : f32, outer : f32) -> Self {
        Self { inner : Vec2::splat(inner), outer : Vec2::splat(outer) }
    }

    pub fn outer_width(&self) -> f32 {
        self.outer.x
    }

    pub fn outer_height(&self) -> f32 {
        self.outer.y
    }
}

pub struct Icon {
    kind : Kind,
    hover : AtomicBool,
}

impl Clone for Icon {
    fn clone(&self) -> Self {
        Self {
            kind : self.kind.clone(),
            hover : AtomicBool::new(false),
        }
    }
}

impl Icon {
    pub fn new(symbol : &'static str) -> Self {
        Self { 
            kind : Kind::Phosphor { symbol },
            hover : AtomicBool::new(false),
        }
    }

    pub fn render(&self, ui : &mut egui::Ui, size : &IconSize) -> Response {
        match self.kind {
            Kind::Phosphor { symbol } => {
                let color = if self.hover.load(Ordering::Relaxed) {
                    ui.ctx().style().visuals.strong_text_color()
                } else {
                    ui.ctx().style().visuals.text_color()
                };
                let response = ui.label(egui::RichText::new(symbol).size(size.inner.y).color(color));
                // let response = ui.add_sized(size.outer, Label::new(egui::RichText::new(symbol).size(size.inner.y).color(color)));
                // let response = ui.add(Label::new(egui::RichText::new(symbol).size(size.inner.y).color(color)));
                // let response = ui.label(egui::RichText::new(symbol).size(size).color(color));

                if response.clicked() {
                    println!("ICON CLICKED...");
                }

                // if response.hovered() {
                    // println!("ICON HOVERED");
                // }
                if response.hovered() {
                    self.hover.store(true, Ordering::Relaxed);
                } else {
                    self.hover.store(false, Ordering::Relaxed);
                }
                response
            }
        }
    }
}

macro_rules! phosphor {
    ($symbol:ident) => (
        Icon::new(egui_phosphor::regular::$symbol)
    );
}

#[derive(Clone)]
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
