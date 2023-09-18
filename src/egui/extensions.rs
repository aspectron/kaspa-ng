use crate::imports::*;

pub enum Confirm {
    Ack,
    Nack,
}


pub trait ResponseExtension {
    fn text_edit_submit(&self, ui: &mut Ui) -> bool;
}

impl ResponseExtension for Response {
    fn text_edit_submit(&self, ui: &mut Ui) -> bool {
        self.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
    }
}

pub trait UiExtension {
    fn medium_button(&mut self, text: impl Into<WidgetText>) -> Response {
        self.medium_button_enabled(true, text)
    }
    fn medium_button_enabled(&mut self, enabled: bool, text: impl Into<WidgetText>) -> Response;
    fn large_button(&mut self, text: impl Into<WidgetText>) -> Response {
        self.large_button_enabled(true, text)
    }
    fn large_button_enabled(&mut self, enabled: bool, text: impl Into<WidgetText>) -> Response;
    fn confirm_medium(&mut self, align : Align, ack : impl Into<WidgetText>, nack : impl Into<WidgetText>) -> Option<Confirm>;
    fn confirm_medium_apply_cancel(&mut self, align : Align) -> Option<Confirm>;
    //  {
    //     self.confirm_medium(
    //         align,
    //         icon_with_text(self, egui_phosphor::light::CHECK, "Apply"),
    //         icon_with_text(self, egui_phosphor::light::X,"Cancel")
    //     )
    // }
}

impl UiExtension for Ui {
    // fn medium_button(&mut self, text: impl Into<WidgetText>) -> Response {
    //     self.add_sized(theme().medium_button_size(), Button::new(text))
    // }
    fn medium_button_enabled(&mut self, enabled: bool, text: impl Into<WidgetText>) -> Response {
        self.add_enabled(
            enabled,
            Button::new(text).min_size(theme().medium_button_size()),
        )
    }
    // fn large_button(&mut self, text: impl Into<WidgetText>) -> Response {
    //     self.add_sized(theme().large_button_size(), Button::new(text))
    // }
    fn large_button_enabled(&mut self, enabled: bool, text: impl Into<WidgetText>) -> Response {
        self.add_enabled(
            enabled,
            Button::new(text).min_size(theme().large_button_size()),
        )
    }

    fn confirm_medium(&mut self, align : Align, ack : impl Into<WidgetText>, nack : impl Into<WidgetText>) -> Option<Confirm> {
        let mut resp: Option<Confirm> = None;
        self.horizontal(|ui| {

            if matches!(align,Align::Max) {
                ui.add_space(ui.available_width() - 16. - (theme().medium_button_size.x + ui.spacing().item_spacing.x)*2.);
            }

            if ui.medium_button(ack).clicked() {
                resp.replace(Confirm::Ack);
            }
            
            if ui.medium_button(nack).clicked() {
                resp.replace(Confirm::Nack);
            }

        });

        resp
    }

    fn confirm_medium_apply_cancel(&mut self, align : Align) -> Option<Confirm> {
        let theme = theme();

        self.confirm_medium(
            align,
            format!("{} {}", egui_phosphor::light::CHECK, "Apply"),
            format!("{} {}", egui_phosphor::light::X, "Cancel"),
            // icon_with_text(self, egui_phosphor::light::CHECK, theme.ack_color, "Apply"),
            // icon_with_text(self, egui_phosphor::light::X,theme.nack_color, "Cancel")
        )
    }

}
