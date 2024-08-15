use egui::*;
use std::hash::Hash;

trait UILayoutExt {
    //fn layout_with_max_rect<R>(&mut self, max_rect:Rect, layout:Layout, add_contents: impl FnOnce(&mut Ui) -> R)->InnerResponse<R>;
    fn indent_with_size<'c, R>(
        &mut self,
        id_source: impl Hash,
        indent: f32,
        add_contents: Box<dyn FnOnce(&mut Ui) -> R + 'c>,
    ) -> InnerResponse<R>;
}

impl UILayoutExt for Ui {
    // fn layout_with_max_rect<R>(&mut self, max_rect:Rect, layout:Layout, add_contents: impl FnOnce(&mut Ui) -> R)->InnerResponse<R> {
    //     let mut child_ui = self.child_ui(max_rect, layout);
    //     let inner = add_contents(&mut child_ui);
    //     let rect = child_ui.min_rect();
    //     let id = self.advance_cursor_after_rect(rect);

    //     InnerResponse::new(inner, self.interact(rect, id, Sense::hover()))

    // }

    fn indent_with_size<'c, R>(
        &mut self,
        id_source: impl Hash,
        indent: f32,
        add_contents: Box<dyn FnOnce(&mut Ui) -> R + 'c>,
    ) -> InnerResponse<R> {
        assert!(
            self.layout().is_vertical(),
            "You can only indent vertical layouts, found {:?}",
            self.layout()
        );

        let mut child_rect = self.available_rect_before_wrap();
        child_rect.min.x += indent;

        let mut child_ui = self.child_ui_with_id_source(child_rect, *self.layout(), id_source);
        let ret = add_contents(&mut child_ui);

        // let left_vline = self.visuals().indent_has_left_vline;
        // let end_with_horizontal_line = self.spacing().indent_ends_with_horizontal_line;

        // if left_vline || end_with_horizontal_line {
        //     if end_with_horizontal_line {
        //         child_ui.add_space(4.0);
        //     }

        //     let stroke = self.visuals().widgets.noninteractive.bg_stroke;
        //     let left_top = child_rect.min - 0.5 * indent * Vec2::X;
        //     let left_top = self.painter().round_pos_to_pixels(left_top);
        //     let left_bottom = pos2(left_top.x, child_ui.min_rect().bottom() - 2.0);
        //     let left_bottom = self.painter().round_pos_to_pixels(left_bottom);

        //     if left_vline {
        //         // draw a faint line on the left to mark the indented section
        //         self.painter.line_segment([left_top, left_bottom], stroke);
        //     }

        //     if end_with_horizontal_line {
        //         let fudge = 2.0; // looks nicer with button rounding in collapsing headers
        //         let right_bottom = pos2(child_ui.min_rect().right() - fudge, left_bottom.y);
        //         self.painter
        //             .line_segment([left_bottom, right_bottom], stroke);
        //     }
        // }

        let response = self.allocate_rect(child_ui.min_rect(), Sense::hover());
        InnerResponse::new(ret, response)
    }
}

type UiBuilderFn = Box<dyn FnOnce(&'_ mut Ui)>;
type FooterUiBuilderFn<V> = Box<dyn FnOnce(&'_ mut Ui, &'_ mut V)>;

pub struct SelectionPanel<V> {
    pub title: WidgetText,
    pub sub: WidgetText,
    pub value: V,
    pub build_heading: Option<UiBuilderFn>,
    pub build_footer: Option<UiBuilderFn>,
}

impl<Value: PartialEq> SelectionPanel<Value> {
    pub fn new(value: Value, title: impl Into<WidgetText>, sub: impl Into<WidgetText>) -> Self {
        Self {
            title: title.into(),
            sub: sub.into(),
            value,
            build_heading: None,
            build_footer: None,
        }
    }
    pub fn heading(mut self, build_heading: impl FnOnce(&mut Ui) + 'static) -> Self {
        self.build_heading = Some(Box::new(build_heading));
        self
    }
    pub fn footer(mut self, build_footer: impl FnOnce(&mut Ui) + 'static) -> Self {
        self.build_footer = Some(Box::new(build_footer));
        self
    }

    pub fn render(
        self,
        ui: &mut Ui,
        bg_color: Color32,
        width: f32,
        min_height: &mut f32,
        current_value: &mut Value,
    ) -> Response {
        let selected = *current_value == self.value;
        let visuals = ui.visuals();
        let selected_bg = visuals.selection.bg_fill;
        let hover_stroke = Stroke::new(1.0, visuals.text_color()); //visuals.window_stroke;
        let frame = Frame::none()
            .stroke(Stroke::new(1.0, Color32::TRANSPARENT))
            .fill(if selected { selected_bg } else { bg_color });
        let mut prepared = frame.begin(ui);

        let add_contents = |ui: &mut Ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(width, ui.available_height()),
                Layout::top_down(Align::Center),
                |ui| {
                    ui.label(" ");
                    ui.label(self.title.strong().heading());
                    ui.label(self.sub);
                    if let Some(build) = self.build_heading {
                        (build)(ui);
                    }
                    let icon = if selected {
                        egui_phosphor::bold::CHECK
                    } else {
                        egui_phosphor::bold::DOT_OUTLINE
                    };
                    ui.label(RichText::new(icon).heading());
                    if let Some(build) = self.build_footer {
                        //ui.visuals_mut().override_text_color = Some(Color32::WHITE);
                        (build)(ui);
                    }
                    ui.label(" ");
                },
            )
            .response
        };

        let _res = add_contents(&mut prepared.content_ui);
        *min_height = min_height.max(prepared.content_ui.min_rect().height());
        prepared.content_ui.set_min_height(*min_height);
        let rect = prepared
            .frame
            .inner_margin
            .expand_rect(prepared.content_ui.min_rect());
        if !selected && ui.allocate_rect(rect, Sense::hover()).hovered() {
            //prepared.frame = prepared.frame.stroke(hover_stroke);
            prepared.frame = prepared.frame.fill(selected_bg).stroke(hover_stroke);
        }

        let res = prepared.end(ui);

        let mut response = res.interact(Sense::click());
        if response.clicked() && *current_value != self.value {
            *current_value = self.value;
            response.mark_changed();
        }
        response
    }
}

pub struct SelectionPanels<V> {
    pub title: WidgetText,
    pub panel_min_width: f32,
    pub panel_max_width: f32,
    pub panels: Vec<SelectionPanel<V>>,
    pub build_footer: FooterUiBuilderFn<V>,
    pub panel_min_height: f32,
    pub vertical: bool,
    pub sep_ratio: f32,
}

impl<Value: PartialEq> SelectionPanels<Value> {
    pub fn new(
        panel_min_width: f32,
        panel_max_width: f32,
        title: impl Into<WidgetText>,
        build_footer: impl FnOnce(&mut Ui, &mut Value) + 'static,
    ) -> Self {
        Self {
            title: title.into(),
            panel_min_width,
            panel_max_width,
            build_footer: Box::new(build_footer),
            panels: vec![],
            panel_min_height: 0.,
            vertical: false,
            sep_ratio: 1.0,
        }
    }
    pub fn add(
        mut self,
        value: Value,
        title: impl Into<WidgetText>,
        sub: impl Into<WidgetText>,
    ) -> Self {
        self.panels.push(SelectionPanel::new(value, title, sub));
        self
    }
    pub fn add_with_footer(
        mut self,
        value: Value,
        title: impl Into<WidgetText>,
        sub: impl Into<WidgetText>,
        build_footer: impl FnOnce(&mut Ui) + 'static,
    ) -> Self {
        self.panels
            .push(SelectionPanel::new(value, title, sub).footer(build_footer));
        self
    }
    pub fn panel_min_height(mut self, min_height: f32) -> Self {
        self.panel_min_height = min_height;
        self
    }

    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }
    pub fn sep_ratio(mut self, sep_ratio: f32) -> Self {
        self.sep_ratio = sep_ratio;
        self
    }

    pub fn render(self, ui: &mut Ui, current_value: &mut Value) -> Response {
        let visuals = ui.visuals();
        let sep_ratio = self.sep_ratio;
        let text_color = visuals.text_color();
        let bg_color = visuals.code_bg_color;
        let before_wrap_width = ui.available_rect_before_wrap().width();
        let mut panel_width = self.panel_min_width.max(
            self.panel_max_width
                .min(before_wrap_width / self.panels.len() as f32),
        );
        let vertical = self.vertical || (before_wrap_width < (panel_width + 2.0) * 3.0);
        let panels_width = if vertical {
            panel_width = self
                .panel_min_width
                .max(self.panel_max_width.min(before_wrap_width - 10.0));
            panel_width
        } else {
            let mut width = 0.0;
            let mut available_width = ui.available_rect_before_wrap().width();
            for _ in 0..self.panels.len() {
                if (available_width - 2.0) < panel_width {
                    break;
                }
                available_width -= panel_width;
                width += panel_width;
            }
            width
        };

        let indent = (before_wrap_width - panels_width) / 2.0;

        let add_contents = |ui: &mut Ui| {
            let mut responce = ui.label(" ");
            //ui.visuals_mut().override_text_color = Some(Color32::WHITE);
            {
                let available_width = ui.available_width() - indent;
                let title =
                    self.title
                        .into_galley(ui, Some(true), available_width, TextStyle::Heading);
                let text_indent = (available_width - title.size().x) / 2.0;
                let rect = ui.cursor().translate(Vec2::new(text_indent, 10.0));
                ui.allocate_exact_size(
                    title.size() + Vec2::new(text_indent, 10.0),
                    Sense::focusable_noninteractive(),
                );
                title.paint_with_fallback_color(ui.painter(), rect.min, text_color);
            }

            // ui.label(format!("before_wrap_width: {before_wrap_width}"));
            // ui.label(format!("panel_width: {panel_width}"));
            let _panels_res = ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                let mut min_height = self.panel_min_height;
                let mut first_row = true;
                for (index, panel) in self.panels.into_iter().enumerate() {
                    let rect = ui.available_rect_before_wrap();
                    let mut row_first_item = index == 0;
                    if (index > 0 && vertical) || rect.width() - 2.0 < panel_width {
                        ui.end_row();
                        row_first_item = true;
                        first_row = false;
                    }
                    // left separator
                    if !row_first_item {
                        let Pos2 { x, y } = ui.cursor().min;
                        let height = min_height * sep_ratio;
                        let m = (min_height - height) / 2.0;
                        ui.painter().vline(
                            x,
                            (y + m)..=(y + m + height),
                            Stroke::new(1.0, text_color),
                        );
                    }

                    // top seperator
                    if !first_row {
                        let Pos2 { x, y } = ui.cursor().min;
                        let width = panel_width * sep_ratio;
                        let m = (panel_width - width) / 2.0;
                        ui.painter().hline(
                            (x + m)..=(x + m + width),
                            y,
                            Stroke::new(1.0, text_color),
                        );
                    }
                    responce |=
                        panel.render(ui, bg_color, panel_width, &mut min_height, current_value);
                }
            });

            // let total_width = panels_res.response.rect.width();
            // ui.allocate_ui_with_layout(
            //     egui::vec2(total_width, ui.available_height()),
            //     Layout::top_down(Align::Center),
            //     |ui| {
            //         ui.set_width(total_width);
            //         (self.build_footer)(ui, current_value)
            //     }
            // );

            // ui.label(format!("bottom width {}", b.response.rect.width()));
            // ui.label(format!("ui.min_rect().width() {}", ui.min_rect().width()));
            responce
        };

        let mut response = ui
            .indent_with_size("selection-panels", indent, Box::new(add_contents))
            .response;
        response |= ui
            .vertical_centered(|ui| (self.build_footer)(ui, current_value))
            .response;
        ui.label(" ");
        // ui.label(format!(" vertical: {vertical}"));
        // ui.label(format!("panels_width {}", panels_width));
        response
    }
}
