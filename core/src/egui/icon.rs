use crate::imports::*;
pub use egui_phosphor::*;

#[derive(Clone)]
pub enum Kind {
    Phosphor { symbol: &'static str },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconSize {
    pub inner: Vec2,
    pub outer: Vec2,
}

impl IconSize {
    pub fn new(inner: Vec2) -> Self {
        Self {
            inner,
            outer: inner,
        }
    }

    pub fn with_padding(mut self, padding: Vec2) -> Self {
        self.outer.x += padding.x * 2.;
        self.outer.y += padding.y * 2.;
        self
    }

    pub fn new_square(inner: f32, outer: f32) -> Self {
        Self {
            inner: Vec2::splat(inner),
            outer: Vec2::splat(outer),
        }
    }

    pub fn outer_width(&self) -> f32 {
        self.outer.x
    }

    pub fn outer_height(&self) -> f32 {
        self.outer.y
    }
}

pub struct Icon {
    kind: Kind,
    hover: AtomicBool,
}

impl Clone for Icon {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind.clone(),
            hover: AtomicBool::new(false),
        }
    }
}

impl Icon {
    pub fn new(symbol: &'static str) -> Self {
        Self {
            kind: Kind::Phosphor { symbol },
            hover: AtomicBool::new(false),
        }
    }

    // pub fn render(&self, ui : &mut egui::Ui, size : f32, color: Color32) -> Response {
    pub fn render(&self, ui: &mut egui::Ui, size: &IconSize, color: Color32) -> Response {
        match self.kind {
            Kind::Phosphor { symbol } => {
                // let response = ui.add(Label::new(egui::RichText::new(symbol).size(size.inner.y).color(color)).sense(Sense::click()));
                // ui.add(Label::new(egui::RichText::new(symbol).size(size).color(color)).sense(Sense::click()))
                ui.add(
                    Label::new(egui::RichText::new(symbol).size(size.inner.y).color(color))
                        .sense(Sense::click()),
                )
            }
        }
    }

    pub fn render_with_options(
        &self,
        ui: &mut egui::Ui,
        size: &IconSize,
        active: bool,
    ) -> Response {
        match self.kind {
            Kind::Phosphor { symbol } => {
                let color = if !active {
                    ui.ctx().style().visuals.widgets.noninteractive.text_color()
                    // ui.ctx().style().visuals.noninteractive().text_color()
                } else if self.hover.load(Ordering::Relaxed) {
                    ui.ctx().style().visuals.widgets.hovered.text_color()
                    // ui.ctx().style().visuals.strong_text_color()
                } else {
                    ui.ctx().style().visuals.widgets.inactive.text_color()
                };
                let response = ui.add(
                    Label::new(egui::RichText::new(symbol).size(size.inner.y).color(color))
                        .sense(Sense::click()),
                );
                // let response = ui.add_sized(size.outer, Label::new(egui::RichText::new(symbol).size(size.inner.y).color(color)));
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
    ($symbol:ident) => {
        Icon::new(egui_phosphor::thin::$symbol)
    };
}

#[derive(Clone)]
pub struct Icons {
    pub back: Icon,
    pub close: Icon,
    pub error: Icon,
    pub gear: Icon,
    pub list: Icon,
    pub vertical_dots: Icon,
    pub vertical_dots_outline: Icon,
    pub faders: Icon,
    pub sliders: Icon,
}

impl Default for Icons {
    fn default() -> Self {
        Self {
            back: phosphor!(ARROW_BEND_UP_LEFT),
            close: phosphor!(X),
            error: phosphor!(SEAL_WARNING),
            gear: phosphor!(GEAR),
            list: phosphor!(LIST),
            vertical_dots: phosphor!(DOTS_THREE_VERTICAL),
            vertical_dots_outline: phosphor!(DOTS_THREE_OUTLINE_VERTICAL),
            faders: phosphor!(FADERS),
            // sliders: phosphor!(SLIDERS_HORIZONTAL),
            sliders: phosphor!(SLIDERS),
        }
    }
}

pub fn icons() -> &'static Icons {
    static ICONS: OnceLock<Icons> = OnceLock::new();
    ICONS.get_or_init(Icons::default)
}
