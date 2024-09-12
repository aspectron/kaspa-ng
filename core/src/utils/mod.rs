use crate::imports::*;

mod average;
pub use average::*;
mod qr;
pub use qr::*;
mod i18n;
#[cfg(not(target_arch = "wasm32"))]
pub use i18n::*;
mod math;
pub use math::*;
mod parse;
pub use parse::*;
mod format;
pub use format::*;
mod arglist;
pub use arglist::*;
mod color;
pub use color::*;
mod image;
pub use image::*;
mod version;
pub use version::*;
mod secret;
pub use secret::*;
mod mnemonic;
pub use mnemonic::*;
mod wallet;
pub use wallet::*;

pub fn is_mobile() -> bool {
    use workflow_core::runtime::{is_android, is_ios};
    is_ios() || is_android()
}

#[macro_export]
macro_rules! spawn {
    ($args: expr) => {{
        let id = concat!(file!(), ":", line!());
        let payload = Payload::new(id);
        if !payload.is_pending() {
            spawn_with_result(&payload, $args);
        }
        payload.take()
    }};
}

pub use spawn;

pub fn icon_with_text(ui: &Ui, icon: &str, color: Color32, text: &str) -> LayoutJob {
    let text_color = ui.ctx().style().visuals.widgets.inactive.text_color(); //.text_color();
    let text_size = ui
        .ctx()
        .style()
        .text_styles
        .get(&TextStyle::Button)
        .unwrap()
        .size;

    let _theme = theme();

    let mut job = LayoutJob {
        halign: Align::Min,
        // justify: true,
        ..Default::default()
    };

    job.append(
        icon,
        0.0,
        TextFormat {
            font_id: FontId::new(text_size + 4., FontFamily::Proportional),
            color,
            valign: Align::Center,
            ..Default::default()
        },
    );

    job.append(
        text,
        2.0,
        TextFormat {
            font_id: FontId::new(text_size, FontFamily::Proportional),
            color: text_color,
            valign: Align::Center,
            ..Default::default()
        },
    );

    job
}

type Handler<'layout, Context> = Box<dyn FnOnce(&mut Context) + 'layout>;

#[derive(Default)]
pub struct CenterLayoutBuilder<'layout, W, Context>
where
    W: Widget,
{
    pub list: Vec<(bool, W, Handler<'layout, Context>)>,
}

impl<'layout, W, Context> CenterLayoutBuilder<'layout, W, Context>
where
    W: Widget,
{
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }
    pub fn add(mut self, widget: W, handler: impl FnOnce(&mut Context) + 'layout) -> Self {
        self.list.push((true, widget, Box::new(handler)));
        self
    }
    pub fn add_enabled(
        mut self,
        enabled: bool,
        widget: W,
        handler: impl FnOnce(&mut Context) + 'layout,
    ) -> Self {
        self.list.push((enabled, widget, Box::new(handler)));
        self
    }

    pub fn build(self, ui: &mut Ui, context: &mut Context) {
        let button_size = theme_style().medium_button_size();
        let available_width = ui.available_width();
        let buttons_len = self.list.len() as f32;
        let spacing = ui.spacing().item_spacing.x;
        let total_width = buttons_len * button_size.x + spacing * (buttons_len - 1.0);
        let margin = (available_width - total_width) * 0.5;

        ui.horizontal(|ui| {
            ui.add_space(margin);
            self.list
                .into_iter()
                .for_each(|(enabled, widget, handler)| {
                    if ui.add_enabled(enabled, widget).clicked() {
                        handler(context);
                    }
                });
        });
    }
}
