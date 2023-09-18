use crate::imports::*;

#[derive(Clone)]
pub enum Kind {
    Phosphor { symbol: &'static str },
}

pub struct Button {
    kind: Kind,
    // icon : Option<String>,
    text : String,
    hover: AtomicBool,
}

impl Clone for Button {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind.clone(),
            text : self.text.clone(),
            // icon : self.icon.clone(),
            hover: AtomicBool::new(false),
        }
    }
}

impl Button {
    pub fn new(symbol: &'static str, text : &str) -> Self {
        Self {
            kind: Kind::Phosphor { symbol },
            text : text.to_string(),
            // icon : Some(icon.to_string()),
            hover: AtomicBool::new(false),
        }
    }

    // pub fn render(&self, ui : &mut egui::Ui, size : f32, color: Color32) -> Response {
    pub fn render(&self, ui: &mut egui::Ui, size: &IconSize) -> Response {
        match self.kind {
            Kind::Phosphor { symbol } => {
                // let response = ui.add(Label::new(egui::RichText::new(symbol).size(size.inner.y).color(color)).sense(Sense::click()));
                // ui.add(Label::new(egui::RichText::new(symbol).size(size).color(color)).sense(Sense::click()))
                ui.add(
                    egui::Button::new(egui::RichText::new(symbol).size(size.inner.y))
                        // .sense(Sense::click()),
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
                    ui.ctx().style().visuals.widgets.noninteractive.text_color() //.text_color();
                    // ui.ctx().style().visuals.widgets.inactive.text_color();//.text_color();

                    // ui.ctx().style().visuals.noninteractive().text_color()
                } else if self.hover.load(Ordering::Relaxed) {
                    ui.ctx().style().visuals.widgets.hovered.text_color() //.text_color();

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


