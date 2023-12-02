use crate::imports::*;

type PopupHandler<'panel> = Box<dyn FnOnce(&mut Ui) + 'panel>;

pub struct PopupPanel<'panel> {
    id: Id,
    title: String,
    min_width: Option<f32>,
    max_height: Option<f32>,
    content: Option<PopupHandler<'panel>>,
    with_caption: bool,
    with_close_button: bool,
    with_pulldown_marker: bool,
    close_on_interaction: bool,
}

impl<'panel> PopupPanel<'panel> {
    pub fn new(
        ui: &mut Ui,
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl FnOnce(&mut Ui) + 'panel,
    ) -> Self {
        let id = ui.make_persistent_id(id.into());

        Self {
            id,
            title: title.into(),
            min_width: None,
            max_height: None,
            content: Some(Box::new(content)),
            with_caption: false,
            with_close_button: false,
            with_pulldown_marker: false,
            close_on_interaction: false,
        }
    }

    pub fn with_min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    pub fn with_max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }

    pub fn with_caption(mut self, caption: bool) -> Self {
        self.with_caption = caption;
        self
    }

    pub fn with_close_button(mut self, close_button: bool) -> Self {
        self.with_close_button = close_button;
        self
    }

    pub fn with_pulldown_marker(mut self, pulldown_marker: bool) -> Self {
        self.with_pulldown_marker = pulldown_marker;
        self
    }

    pub fn with_close_on_interaction(mut self, close_on_interaction: bool) -> Self {
        self.close_on_interaction = close_on_interaction;
        self
    }

    pub fn build(&mut self, ui: &mut Ui) {
        let title = self.title.clone();
        let content = self.content.take().unwrap();
        // let response = ui.add(Label::new(format!("{} ⏷", title)).sense(Sense::click()));
        let text = if self.with_pulldown_marker {
            format!("{} ⏷", title)
        } else {
            title.clone()
        };

        let response = ui.add(Label::new(text).sense(Sense::click()));

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
            AboveOrBelow::Below,
            self.close_on_interaction,
            |ui| {
                if let Some(width) = self.min_width {
                    ui.set_min_width(width);
                }

                if let Some(height) = self.max_height {
                    ui.set_max_height(height);
                }

                if self.with_caption {
                    ui.horizontal(|ui| {
                        ui.label(title);

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
                    ui.space();
                }

                content(ui);

                if self.with_close_button {
                    ui.space();
                    ui.separator();

                    ui.add_space(8.);
                    ui.vertical_centered(|ui| {
                        if ui.medium_button("Close").clicked() {
                            ui.memory_mut(|mem| mem.close_popup());
                        }
                    });
                    ui.add_space(8.);
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
        } else if ui.input(|i| i.key_pressed(Key::Escape)) || widget_response.clicked_elsewhere() {
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
