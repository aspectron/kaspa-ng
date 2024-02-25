use crate::imports::*;

type PopupWidget<'panel> = Box<dyn FnOnce(&mut Ui) -> Response + 'panel>;
type PopupHandler<'panel> = Box<dyn FnOnce(&mut Ui, &mut bool) + 'panel>;

pub struct PopupPanel<'panel> {
    id: Id,
    min_width: Option<f32>,
    max_height: Option<f32>,
    widget: PopupWidget<'panel>,
    content: PopupHandler<'panel>,
    caption: Option<String>,
    with_close_button: bool,
    close_on_interaction: bool,
    close_on_escape: bool,
    above_or_below: AboveOrBelow,
    with_padding: bool,
}

impl<'panel> PopupPanel<'panel> {
    pub fn id(ui: &mut Ui, id: impl Into<String>) -> Id {
        ui.make_persistent_id(id.into())
    }

    pub fn new(
        id: Id,
        widget: impl FnOnce(&mut Ui) -> Response + 'panel,
        content: impl FnOnce(&mut Ui, &mut bool) + 'panel,
    ) -> Self {
        // let id = ui.make_persistent_id(id.into());

        Self {
            // id : id.into(),
            id,
            min_width: None,
            max_height: None,
            widget: Box::new(widget),
            content: Box::new(content),
            caption: None,
            with_close_button: false,
            close_on_interaction: false,
            close_on_escape: true,
            above_or_below: AboveOrBelow::Below,
            with_padding: true,
        }
    }

    // pub fn new_with_string_id(
    //     ui: &mut Ui,
    //     id: impl Into<String>,
    //     // id: Id,
    //     widget: impl FnOnce(&mut Ui) -> Response + 'panel,
    //     content: impl FnOnce(&mut Ui, &mut bool) + 'panel,
    // ) -> Self {
    //     let id = ui.make_persistent_id(id.into());

    //     Self {
    //         id,
    //         min_width: None,
    //         max_height: None,
    //         widget: Box::new(widget),
    //         content: Box::new(content),
    //         caption: None,
    //         with_close_button: false,
    //         close_on_interaction: false,
    //         close_on_escape: true,
    //         above_or_below: AboveOrBelow::Below,
    //         with_padding: true,
    //     }
    // }

    pub fn is_open(ui: &mut Ui, popup_id: Id) -> bool {
        ui.memory(|mem| mem.is_popup_open(popup_id))
    }

    pub fn with_min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    pub fn with_max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }

    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn with_close_button(mut self, close_button: bool) -> Self {
        self.with_close_button = close_button;
        self
    }

    pub fn with_close_on_interaction(mut self, close_on_interaction: bool) -> Self {
        self.close_on_interaction = close_on_interaction;
        self
    }

    pub fn with_above_or_below(mut self, above_or_below: AboveOrBelow) -> Self {
        self.above_or_below = above_or_below;
        self
    }

    pub fn with_padding(mut self, with_padding: bool) -> Self {
        self.with_padding = with_padding;
        self
    }

    pub fn build(self, ui: &mut Ui) {
        let response = (self.widget)(ui);
        if response.clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(self.id));
        }

        // replica of [`egui::popup::popup_above_or_below_widget`] that
        // ignores clicks inside of its area allowing the panel to
        // persist while the user interacts with it and closing
        // once triggered via `mem.close_popup()` or clicking outside of it.
        popup_above_or_below_widget_local(
            ui,
            self.id,
            &response,
            self.above_or_below,
            self.close_on_interaction,
            self.close_on_escape,
            |ui| {
                if let Some(width) = self.min_width {
                    ui.set_min_width(width);
                }

                if let Some(height) = self.max_height {
                    ui.set_max_height(height);
                }

                if let Some(caption) = self.caption {
                    ui.horizontal(|ui| {
                        ui.label(caption);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            use egui_phosphor::light::X;
                            if ui
                                .add(Label::new(RichText::new(X).size(16.)).sense(Sense::click()))
                                .clicked()
                            {
                                ui.memory_mut(|mem| mem.close_popup());
                            }
                        });
                    });

                    ui.separator();
                    if self.with_padding {
                        ui.space();
                    }
                }

                let mut close_popup = false;
                (self.content)(ui, &mut close_popup);

                if self.with_close_button {
                    if self.with_padding {
                        ui.space();
                    }
                    ui.separator();

                    ui.add_space(8.);
                    ui.vertical_centered(|ui| {
                        if ui.medium_button(i18n("Close")).clicked() {
                            close_popup = true;
                        }
                    });
                    ui.add_space(8.);
                }

                if close_popup {
                    ui.memory_mut(|mem| mem.close_popup());
                }
            },
        );
    }
}

pub fn popup_above_or_below_widget_local<R>(
    ui: &Ui,
    popup_id: Id,
    widget_response: &Response,
    above_or_below: AboveOrBelow,
    close_on_interaction: bool,
    close_on_escape: bool,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    if ui.memory(|mem| mem.is_popup_open(popup_id)) {
        let (pos, pivot) = match above_or_below {
            AboveOrBelow::Above => (widget_response.rect.left_top(), Align2::LEFT_BOTTOM),
            AboveOrBelow::Below => (widget_response.rect.left_bottom(), Align2::LEFT_TOP),
        };

        let inner = Area::new(popup_id)
            .order(Order::Foreground)
            .constrain(true)
            .fixed_pos(pos)
            .pivot(pivot)
            .show(ui.ctx(), |ui| {
                // Note: we use a separate clip-rect for this area, so the popup can be outside the parent.
                // See https://github.com/emilk/egui/issues/825
                let frame = Frame::popup(ui.style());
                let frame_margin = frame.total_margin();
                frame
                    .show(ui, |ui| {
                        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                            ui.set_width(widget_response.rect.width() - frame_margin.sum().x);
                            add_contents(ui)
                        })
                        .inner
                    })
                    .inner
            });

        let mut close_popup = false;
        if close_on_interaction {
            if ui.input(|i| i.key_pressed(Key::Escape)) || widget_response.clicked_elsewhere() {
                close_popup = true;
            }
        } else if close_on_escape
            && (ui.input(|i| i.key_pressed(Key::Escape)) || widget_response.clicked_elsewhere())
        {
            let response = inner.response;
            ui.ctx().input(|i| {
                let pointer = &i.pointer;
                if pointer.any_click() {
                    if let Some(pos) = pointer.interact_pos() {
                        if !response.rect.contains(pos) {
                            close_popup = true;
                        }
                    }
                }
            });
        }

        if close_popup {
            ui.memory_mut(|mem| mem.close_popup());
        }

        Some(inner.inner)
    } else {
        None
    }
}
