use crate::imports::*;

pub trait ResponseExtension {
    fn text_edit_submit(&self, ui: &mut Ui) -> bool;
}

impl ResponseExtension for Response {
    fn text_edit_submit(&self, ui: &mut Ui) -> bool {
        self.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
    }
}

pub trait UiExtension {
    fn large_button(&mut self, text: impl Into<WidgetText>) -> Response;
    fn large_button_enabled(&mut self, enabled: bool, text: impl Into<WidgetText>) -> Response;
}

impl UiExtension for Ui {
    fn large_button(&mut self, text: impl Into<WidgetText>) -> Response {
        self.add_sized(theme().large_button_size(), Button::new(text))
    }
    fn large_button_enabled(&mut self, enabled: bool, text: impl Into<WidgetText>) -> Response {
        self.add_enabled(
            enabled,
            Button::new(text).min_size(theme().large_button_size()),
        )
    }
}
