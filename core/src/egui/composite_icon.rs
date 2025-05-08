//use egui::widget_text::WidgetTextGalley;
use egui::*;
use std::sync::Arc;

pub struct LayoutResult {
    pos: Pos2,
    text_pos: Pos2,
    response: Response,
    icon_text: Arc<Galley>,
    text: Option<Arc<Galley>>,
    secondary_text: Option<Arc<Galley>>,
}
impl LayoutResult {
    fn new(
        pos: Pos2,
        text_pos: Pos2,
        response: Response,
        icon_text: Arc<Galley>,
        text: Option<Arc<Galley>>,
        secondary_text: Option<Arc<Galley>>,
    ) -> Self {
        Self {
            pos,
            text_pos,
            response,
            icon_text,
            text,
            secondary_text,
        }
    }
}

/// Clickable button with text.
///
/// See also [`Ui::button`].
///
/// ```ignore
/// # egui::__run_test_ui(|ui| {
/// # fn do_stuff() {}
///
/// if ui.add(CompositeIcon::new(egui_phosphor::bold::ARROW_BEND_UP_LEFT).clicked() {
///     do_stuff();
/// }
///
/// // A greyed-out and non-interactive button:
/// if ui.add_enabled(false, CompositeIcon::new(egui_phosphor::bold::ARROW_BEND_UP_LEFT).clicked() {
///     unreachable!();
/// }
/// # });
/// ```
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct CompositeIcon {
    icon: RichText,
    text: Option<WidgetText>,
    secondary_text: Option<WidgetText>,

    /// None means default for interact
    fill: Option<Color32>,
    stroke: Option<Stroke>,
    sense: Sense,
    small: bool,
    frame: Option<bool>,
    rounding: Option<CornerRadius>,
    padding: Option<Vec2>,
    selected: bool,
    with_frame: bool,
    icon_size: f32,
}

impl CompositeIcon {
    pub fn new(icon: impl Into<RichText>) -> Self {
        Self::opt_icon_and_text(icon, Option::<String>::None, Option::<String>::None)
    }
    pub fn secondary_text(mut self, text: impl Into<WidgetText>) -> Self {
        self.secondary_text = Some(text.into());
        self
    }
    pub fn text(mut self, text: impl Into<WidgetText>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn opt_icon_and_text(
        icon: impl Into<RichText>,
        text: Option<impl Into<WidgetText>>,
        secondary_text: Option<impl Into<WidgetText>>,
    ) -> Self {
        Self {
            text: text.map(|a| a.into()),
            secondary_text: secondary_text.map(|a| a.into()),
            icon: icon.into(),
            fill: None,
            stroke: None,
            sense: Sense::click(),
            small: false,
            frame: None,
            rounding: None,
            padding: None,
            selected: false,
            with_frame: false,
            icon_size: 30.0,
        }
    }

    pub fn padding(mut self, padding: Option<Vec2>) -> Self {
        self.padding = padding;
        self
    }

    /// Override background fill color. Note that this will override any on-hover effects.
    /// Calling this will also turn on the frame.
    pub fn fill(mut self, fill: impl Into<Color32>) -> Self {
        self.fill = Some(fill.into());
        self.frame = Some(true);
        self
    }

    /// Override button stroke. Note that this will override any on-hover effects.
    /// Calling this will also turn on the frame.
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = Some(stroke.into());
        self.frame = Some(true);
        self
    }

    /// Make this a small button, suitable for embedding into text.
    pub fn small(mut self) -> Self {
        if let Some(text) = self.text {
            self.text = Some(text.text_style(TextStyle::Body));
        }
        self.small = true;
        self
    }

    /// Turn off the frame
    pub fn frame(mut self, frame: bool) -> Self {
        self.frame = Some(frame);
        self
    }

    /// By default, buttons senses clicks.
    /// Change this to a drag-button with `Sense::drag()`.
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }

    /// Set the rounding of the button.
    pub fn rounding(mut self, rounding: impl Into<CornerRadius>) -> Self {
        self.rounding = Some(rounding.into());
        self
    }

    /// If `true`, mark this button as "selected".
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// If `true`, mark this icon as "frame button".
    pub fn with_frame(mut self, frame: bool) -> Self {
        self.with_frame = frame;
        self
    }

    pub fn icon_size(mut self, icon_size: f32) -> Self {
        self.icon_size = icon_size;
        self
    }

    fn _padding(&self, ui: &Ui) -> Vec2 {
        let frame = self.frame.unwrap_or_else(|| ui.visuals().button_frame);

        let mut button_padding = self.padding.unwrap_or(if frame && self.with_frame {
            ui.spacing().button_padding
        } else {
            Vec2::ZERO
        });

        if self.small {
            button_padding.y = 0.0;
        }

        button_padding
    }

    /// Do layout and position the galley in the ui, without painting it or adding widget info.
    pub fn layout_in_ui(&self, ui: &mut Ui) -> LayoutResult {
        let sense = {
            // We only want to focus icon if the screen reader is on.
            if ui.memory(|mem| mem.options.screen_reader) {
                Sense::focusable_noninteractive()
            } else {
                self.sense
            }
        };

        let padding = self._padding(ui);

        let mut text_size = Vec2::ZERO;
        let text_wrap_width = ui.available_width() - 2.0 * padding.x;
        let mut secondary_text_style = TextStyle::Name("CompositeButtonSubtext".into());
        if !ui.style().text_styles.contains_key(&secondary_text_style) {
            secondary_text_style = TextStyle::Body;
        }

        let text = self.text.clone().map(|text| {
            text.into_galley(
                ui,
                Some(TextWrapMode::Wrap),
                text_wrap_width,
                TextStyle::Button,
            )
        });
        let secondary_text = self.secondary_text.clone().map(|text| {
            text.into_galley(
                ui,
                Some(TextWrapMode::Wrap),
                text_wrap_width,
                secondary_text_style,
            )
        });

        if let Some(text) = &text {
            text_size += text.size();
            text_size.y += ui.spacing().item_spacing.y;
        }
        if let Some(secondary_text) = &secondary_text {
            text_size.y += ui.spacing().item_spacing.y + secondary_text.size().y;
            text_size.x += secondary_text.size().x.max(text_size.x);
        }

        let create_text_pos = |rect: Rect| {
            pos2(
                rect.left() + padding.x,
                rect.top() + rect.size().y - text_size.y - padding.y,
            )
        };

        let create_result = |mut pos: Pos2,
                             _icon_size: Vec2,
                             response: Response,
                             text_galley,
                             text,
                             secondary_text| {
            //pos += padding;
            let text_pos = create_text_pos(response.rect);
            pos.y += padding.y;

            // h-center
            //pos.x = response.rect.left() + (response.rect.width() - icon_size.x)/2.0;
            LayoutResult::new(pos, text_pos, response, text_galley, text, secondary_text)
        };

        // if let WidgetText::Galley(galley) = self.icon.clone() {
        //     // If the user said "use this specific galley", then just use it:
        //     let mut size = galley.size();
        //     let icon_size = size;
        //     size.x = text_size.x.max(size.x) + padding.x * 2.0;
        //     size.y += text_size.y + padding.y * 2.0;

        //     let (rect, response) = ui.allocate_exact_size(size, sense);
        //     let pos = match galley.job.halign {
        //         Align::LEFT => rect.left_top(),
        //         Align::Center => rect.center_top(),
        //         Align::RIGHT => rect.right_top(),
        //     };
        //     let text_galley = WidgetTextGalley {
        //         galley,
        //         galley_has_color: true,
        //     };
        //     return create_result(pos, icon_size, response, text_galley, text, secondary_text);
        // }

        let valign = ui.layout().vertical_align();
        let mut layout_job = WidgetText::from(self.icon.clone().size(self.icon_size))
            .into_layout_job(ui.style(), FontSelection::Default, valign);

        let truncate = true;
        let wrap = !truncate && ui.wrap_mode() == TextWrapMode::Wrap;
        let available_width = ui.available_width();

        if wrap
            && ui.layout().main_dir() == Direction::LeftToRight
            && ui.layout().main_wrap()
            && available_width.is_finite()
        {
            // On a wrapping horizontal layout we want text to start after the previous widget,
            // then continue on the line below! This will take some extra work:

            let cursor = ui.cursor();
            let first_row_indentation = available_width - ui.available_size_before_wrap().x;
            assert!(first_row_indentation.is_finite());

            layout_job.wrap.max_width = available_width;
            layout_job.first_row_min_height = cursor.height();
            layout_job.halign = Align::Min;
            layout_job.justify = false;
            if let Some(first_section) = layout_job.sections.first_mut() {
                first_section.leading_space = first_row_indentation;
            }
            let text_galley = ui.fonts(|fonts| fonts.layout_job(layout_job));

            let pos = pos2(ui.max_rect().left(), ui.cursor().top());
            assert!(!text_galley.rows.is_empty(), "Galleys are never empty");
            // collect a response from many rows:
            let mut rect = text_galley.rows[0].rect;
            let mut rect_size = rect.size();
            let icon_size = rect_size;
            rect_size.x = text_size.x.max(rect_size.x) + padding.x * 2.0;
            rect_size.y += text_size.y + padding.y * 2.0;
            rect.set_width(rect_size.x);
            rect.set_height(rect_size.y);

            let rect = rect.translate(vec2(pos.x, pos.y));
            let mut response = ui.allocate_rect(rect, sense);
            for row in text_galley.rows.iter().skip(1) {
                let rect = row.rect.translate(vec2(pos.x, pos.y));
                response |= ui.allocate_rect(rect, sense);
            }

            create_result(pos, icon_size, response, text_galley, text, secondary_text)
        } else {
            if truncate {
                layout_job.wrap.max_width = available_width;
                layout_job.wrap.max_rows = 1;
                layout_job.wrap.break_anywhere = true;
            } else if wrap {
                layout_job.wrap.max_width = available_width;
            } else {
                layout_job.wrap.max_width = f32::INFINITY;
            };

            layout_job.halign = Align::Center;
            layout_job.justify = ui.layout().horizontal_justify();

            let text_galley = ui.fonts(|fonts| fonts.layout_job(layout_job));

            let mut size = text_galley.size();
            let icon_size = size;
            size.x = text_size.x.max(size.x) + padding.x * 2.0;
            size.y += text_size.y + padding.y * 2.0;

            let (rect, response) = ui.allocate_exact_size(size, sense);
            let pos = match text_galley.job.halign {
                Align::LEFT => rect.left_top(),
                Align::Center => rect.center_top(),
                Align::RIGHT => rect.right_top(),
            };

            create_result(pos, icon_size, response, text_galley, text, secondary_text)
        }
    }
}

impl Widget for CompositeIcon {
    fn ui(self, ui: &mut Ui) -> Response {
        let LayoutResult {
            pos,
            text_pos,
            response,
            icon_text,
            text,
            secondary_text,
        } = self.layout_in_ui(ui);

        response.widget_info(|| {
            if let Some(text) = &self.text {
                WidgetInfo::labeled(WidgetType::Button, true, text.text())
            } else {
                WidgetInfo::new(WidgetType::Button)
            }
        });

        let rect = response.rect;

        if ui.is_rect_visible(response.rect) {
            let frame = self.frame.unwrap_or_else(|| ui.visuals().button_frame);
            let visuals = ui.style().interact(&response);

            let (frame_expansion, frame_rounding, frame_fill, frame_stroke) = if self.selected {
                let selection = ui.visuals().selection;
                (
                    Vec2::ZERO,
                    CornerRadius::ZERO,
                    selection.bg_fill,
                    selection.stroke,
                )
            } else if frame {
                let expansion = Vec2::splat(visuals.expansion);
                (
                    expansion,
                    visuals.corner_radius,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                )
            } else {
                Default::default()
            };

            if self.with_frame {
                let frame_rounding = self.rounding.unwrap_or(frame_rounding);
                let frame_fill = self.fill.unwrap_or(frame_fill);
                let frame_stroke = self.stroke.unwrap_or(frame_stroke);
                ui.painter().rect(
                    rect.expand2(frame_expansion),
                    frame_rounding,
                    frame_fill,
                    frame_stroke,
                    StrokeKind::Outside,
                );
            }

            // let override_text_color = if icon_text.job.galley_has_color {
            //     None
            // } else {
            // let override_text_color = Some(
            //     ui.style()
            //         .interact_selectable(&response, self.selected)
            //         .text_color(),
            // );
            //};

            let button_padding = self._padding(ui);
            ui.painter().add(epaint::TextShape {
                pos,
                galley: icon_text,
                override_text_color: None,
                underline: Stroke::NONE,
                angle: 0.0,
                fallback_color: ui
                    .style()
                    .interact_selectable(&response, self.selected)
                    .text_color(),
                opacity_factor: 1.0,
            });

            let mut cursor_y = text_pos.y + ui.spacing().item_spacing.y;

            if let Some(text) = text {
                let mut pos = ui
                    .layout()
                    .align_size_within_rect(text.size(), rect.shrink2(button_padding))
                    .min;
                pos.y = cursor_y;
                cursor_y += text.size().y + ui.spacing().item_spacing.y;

                //text.paint_with_visuals(ui.painter(), pos, visuals);
                ui.painter().galley(pos, text, visuals.text_color());
            }

            if let Some(secondary_text) = secondary_text {
                let mut pos = ui
                    .layout()
                    .align_size_within_rect(secondary_text.size(), rect.shrink2(button_padding))
                    .min;
                pos.y = cursor_y;

                //secondary_text.paint_with_visuals(ui.painter(), pos, visuals);
                ui.painter()
                    .galley(pos, secondary_text, visuals.text_color());
            }
        }

        if let Some(cursor) = ui.visuals().interact_cursor {
            if response.hovered() {
                ui.ctx().set_cursor_icon(cursor);
            }
        }

        response
    }
}
