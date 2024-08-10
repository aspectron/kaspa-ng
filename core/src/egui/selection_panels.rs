use egui::*;

// trait LayoutWithMaxRect{
//     fn layout_with_max_rect<R>(&mut self, max_rect:Rect, layout:Layout, add_contents: impl FnOnce(&mut Ui) -> R)->InnerResponse<R>;
// }

// impl LayoutWithMaxRect for Ui{
//     fn layout_with_max_rect<R>(&mut self, max_rect:Rect, layout:Layout, add_contents: impl FnOnce(&mut Ui) -> R)->InnerResponse<R> {
//         let mut child_ui = self.child_ui(max_rect, layout);
//         let inner = add_contents(&mut child_ui);
//         let rect = child_ui.min_rect();
//         let id = self.advance_cursor_after_rect(rect);

//         InnerResponse::new(inner, self.interact(rect, id, Sense::hover()))

//     }
// }

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
        let selected_bg = Color32::from_rgb(67, 76, 84);
        let hover_stroke_color = Color32::WHITE;
        // let mut rect = ui.cursor();
        // rect.set_width(width);

        // ui.painter().rect(
        //     rect,
        //     Rounding::ZERO,
        //     Color32::GRAY,
        //     Stroke::new(1.0, Color32::GREEN),
        // );

        // let res = ui.layout_with_max_rect(rect, Layout::top_down(Align::Center), |ui|{
        //     ui.label(self.title);
        //     ui.label(self.sub);
        //     if let Some(build) = self.build_heading{
        //         (build)(ui);
        //     }
        //     ui.checkbox(&mut selected, "");
        //     if let Some(build) = self.build_footer{
        //         (build)(ui);
        //     }
        // }).response;
        let frame = Frame::none().fill(if selected { selected_bg } else { bg_color });
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
                    ui.label(RichText::new(icon).heading().color(Color32::WHITE));
                    if let Some(build) = self.build_footer {
                        ui.visuals_mut().override_text_color = Some(Color32::WHITE);
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
        if ui.allocate_rect(rect, Sense::hover()).hovered() {
            prepared.frame = prepared.frame.stroke(Stroke::new(1.0, hover_stroke_color))
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

    pub fn render(self, ui: &mut Ui, current_value: &mut Value) -> Response {
        let frame_bg = Color32::from_rgb(17, 19, 24);
        let text_color = Color32::WHITE;
        let even_panel_bg = Color32::from_rgb(44, 50, 59);
        let odd_panel_bg = Color32::from_rgb(54, 63, 71);

        let frame = Frame::none()
            .fill(frame_bg)
            .rounding(egui::Rounding::same(10.0))
            .stroke(Stroke::NONE);

        let mut prepared = frame.begin(ui);
        let add_contents = |ui: &mut Ui| {
            let mut responce = ui.label(" ");
            //ui.visuals_mut().override_text_color = Some(Color32::WHITE);
            //ui.button(text)
            {
                let title = self.title.into_galley(
                    ui,
                    Some(true),
                    ui.available_width(),
                    TextStyle::Heading,
                );
                let rect = ui.cursor().translate(Vec2::splat(10.0));
                ui.allocate_exact_size(
                    title.size() + Vec2::splat(10.0),
                    Sense::focusable_noninteractive(),
                );
                title.paint_with_fallback_color(ui.painter(), rect.min, text_color);
            }

            let before_wrap_width = ui.available_rect_before_wrap().width();
            let panel_width = self.panel_min_width.max(
                self.panel_max_width
                    .min(before_wrap_width / self.panels.len() as f32),
            );
            let vertical = self.vertical || (before_wrap_width - 2. < panel_width * 3.0);
            // ui.label(format!("before_wrap_width: {before_wrap_width}"));
            // ui.label(format!("panel_width: {panel_width}"));
            let panels_res = ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                let mut min_height = self.panel_min_height;
                for (index, panel) in self.panels.into_iter().enumerate() {
                    let rect = ui.available_rect_before_wrap();
                    if (index > 0 && vertical) || rect.width() - 2.0 < panel_width {
                        ui.end_row();
                    }
                    let bg_color = if index % 2 == 0 {
                        even_panel_bg
                    } else {
                        odd_panel_bg
                    };
                    responce |=
                        panel.render(ui, bg_color, panel_width, &mut min_height, current_value);
                }
            });

            let total_width = panels_res.response.rect.width();
            ui.allocate_ui_with_layout(
                egui::vec2(total_width, ui.available_height()),
                Layout::top_down(Align::Center),
                |ui| (self.build_footer)(ui, current_value),
            );
            ui.label(" ");
            // ui.label(format!(" vertical: {vertical}"));
            // ui.label(format!("panels width {}", total_width));
            // ui.label(format!("bottom width {}", b.response.rect.width()));
            // ui.label(format!("ui.min_rect().width() {}", ui.min_rect().width()));
            responce
        };

        let res = add_contents(&mut prepared.content_ui);
        //prepared.frame = prepared.frame.fill(Color32::GREEN);
        res | prepared.end(ui)
    }
}
