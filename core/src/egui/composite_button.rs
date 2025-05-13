use egui::load::{TextureLoadResult, TexturePoll};
use egui::*;

use super::theme_style;

pub enum Composite<'a> {
    Image(Image<'a>),
    Icon(RichText),
}

impl<'a> Composite<'a> {
    pub fn image(image: Image<'a>) -> Self {
        Self::Image(image)
    }
    pub fn icon(icon: impl Into<RichText>) -> Self {
        Self::Icon(icon.into())
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
/// if ui.add(CompositeButton::new("Click me", "Secondary text")).clicked() {
///     do_stuff();
/// }
///
/// // A greyed-out and non-interactive button:
/// if ui.add_enabled(false, CompositeButton::new("Can't click this", "Secondary text")).clicked() {
///     unreachable!();
/// }
/// # });
/// ```
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct CompositeButton<'a> {
    image: Option<Composite<'a>>,
    icon_size: Option<f32>,
    text: Option<WidgetText>,
    secondary_text: Option<WidgetText>,
    shortcut_text: WidgetText,
    wrap: Option<bool>,

    /// None means default for interact
    fill: Option<Color32>,
    stroke: Option<Stroke>,
    sense: Sense,
    small: bool,
    frame: Option<bool>,
    min_size: Vec2,
    rounding: Option<CornerRadius>,
    padding: Option<Vec2>,
    selected: bool,
    show_loading_spinner: Option<bool>,
    pulldown_selector: bool,
}

impl<'a> CompositeButton<'a> {
    pub fn new(text: impl Into<WidgetText>, secondary_text: impl Into<WidgetText>) -> Self {
        Self::opt_image_and_text(None, Some(text.into()), Some(secondary_text.into()))
    }
    pub fn secondary_text(mut self, text: impl Into<WidgetText>) -> Self {
        self.secondary_text = Some(text.into());
        self
    }
    pub fn text(mut self, text: impl Into<WidgetText>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Creates a button with an image. The size of the image as displayed is defined by the provided size.
    #[allow(clippy::needless_pass_by_value)]
    pub fn image(image: impl Into<Composite<'a>>) -> Self {
        Self::opt_image_and_text(Some(image.into()), None, None)
    }

    /// Creates a button with an image to the left of the text. The size of the image as displayed is defined by the provided size.
    #[allow(clippy::needless_pass_by_value)]
    pub fn image_and_text(
        image: impl Into<Composite<'a>>,
        text: impl Into<WidgetText>,
        secondary_text: impl Into<WidgetText>,
    ) -> Self {
        Self::opt_image_and_text(
            Some(image.into()),
            Some(text.into()),
            Some(secondary_text.into()),
        )
    }

    pub fn opt_image_and_text(
        image: Option<Composite<'a>>,
        text: Option<WidgetText>,
        secondary_text: Option<WidgetText>,
    ) -> Self {
        Self {
            text,
            image,
            icon_size: None,
            shortcut_text: Default::default(),
            wrap: None,
            fill: None,
            stroke: None,
            sense: Sense::click(),
            small: false,
            frame: None,
            min_size: Vec2::ZERO,
            rounding: None,
            // padding: Some(vec2(2.0, 4.0)),
            padding: None,
            selected: false,
            show_loading_spinner: None,
            secondary_text,
            pulldown_selector: false,
        }
    }

    pub fn padding(mut self, padding: Option<Vec2>) -> Self {
        self.padding = padding;
        self
    }

    pub fn icon_size(mut self, icon_size: f32) -> Self {
        self.icon_size = Some(icon_size);
        self
    }

    /// If `true`, the text will wrap to stay within the max width of the [`Ui`].
    ///
    /// By default [`Self::wrap`] will be true in vertical layouts
    /// and horizontal layouts with wrapping,
    /// and false on non-wrapping horizontal layouts.
    ///
    /// Note that any `\n` in the text will always produce a new line.
    #[inline]
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = Some(wrap);
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

    /// Set the minimum size of the button.
    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    /// Set the rounding of the button.
    pub fn rounding(mut self, rounding: impl Into<CornerRadius>) -> Self {
        self.rounding = Some(rounding.into());
        self
    }

    /// Show some text on the right side of the button, in weak color.
    ///
    /// Designed for menu buttons, for setting a keyboard shortcut text (e.g. `Ctrl+S`).
    ///
    /// The text can be created with [`Context::format_shortcut`].
    pub fn shortcut_text(mut self, shortcut_text: impl Into<WidgetText>) -> Self {
        self.shortcut_text = shortcut_text.into();
        self
    }

    /// If `true`, mark this button as "selected".
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Show a spinner when the image is loading.
    ///
    /// By default this uses the value of [`Visuals::image_loading_spinners`].
    #[inline]
    pub fn show_loading_spinner(mut self, show: bool) -> Self {
        self.show_loading_spinner = Some(show);
        self
    }

    #[inline]
    pub fn with_pulldown_selector(mut self, pulldown_selector: bool) -> Self {
        self.pulldown_selector = pulldown_selector;
        self
    }
}

impl Widget for CompositeButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            text,
            image,
            icon_size,
            shortcut_text,
            wrap,
            fill,
            stroke,
            sense,
            small,
            frame,
            min_size,
            rounding,
            padding,
            selected,
            show_loading_spinner,
            secondary_text,
            pulldown_selector,
        } = self;

        let frame = frame.unwrap_or_else(|| ui.visuals().button_frame);

        let mut button_padding = padding.unwrap_or(if frame {
            ui.spacing().button_padding
        } else {
            Vec2::ZERO
        });
        if small {
            button_padding.y = 0.0;
        }

        let pulldown_padding = if pulldown_selector { 20.0 } else { 0.0 };

        let space_available_for_image = if let Some(text) = &text {
            let font_height = ui.fonts(|fonts| font_height(text, fonts, ui.style()));
            Vec2::splat(font_height) // Reasonable?
        } else {
            ui.available_size() - 2.0 * button_padding
        };

        let image_size = if let Some(image) = &image {
            match image {
                Composite::Image(image) => image
                    .load_and_calc_size(ui, space_available_for_image)
                    .unwrap_or(space_available_for_image),
                Composite::Icon(_icon) => icon_size
                    .map(Vec2::splat)
                    .unwrap_or(Vec2::splat(theme_style().composite_icon_size)),
            }
        } else {
            Vec2::ZERO
        };

        let mut text_wrap_width = ui.available_width() - 2.0 * button_padding.x;
        if image.is_some() {
            text_wrap_width -= image_size.x + ui.spacing().icon_spacing;
        }
        if !shortcut_text.is_empty() {
            text_wrap_width -= 60.0; // Some space for the shortcut text (which we never wrap).
        }

        let mut secondary_text_style = TextStyle::Name("CompositeButtonSubtext".into());
        if !ui.style().text_styles.contains_key(&secondary_text_style) {
            secondary_text_style = TextStyle::Monospace;
        }

        let wrap_mode = wrap.map(|wrap| {
            if wrap {
                TextWrapMode::Wrap
            } else {
                TextWrapMode::Extend
            }
        });

        let text =
            text.map(|text| text.into_galley(ui, wrap_mode, text_wrap_width, TextStyle::Button));
        let secondary_text = secondary_text
            .map(|text| text.into_galley(ui, wrap_mode, text_wrap_width, secondary_text_style));
        let shortcut_text = (!shortcut_text.is_empty()).then(|| {
            shortcut_text.into_galley(
                ui,
                Some(TextWrapMode::Extend),
                f32::INFINITY,
                TextStyle::Button,
            )
        });

        let mut desired_size = Vec2::new(pulldown_padding, 0.0); //Vec2::ZERO;
        let mut img_plus_spacing_width = 0.0;
        if image.is_some() {
            desired_size.x += image_size.x;
            img_plus_spacing_width += image_size.x;
            desired_size.y = desired_size.y.max(image_size.y);

            if text.is_some() || secondary_text.is_some() {
                desired_size.x += ui.spacing().icon_spacing;
                img_plus_spacing_width += ui.spacing().icon_spacing;
            }
        }
        let mut text_width = 0.0;
        if let Some(text) = &text {
            desired_size.x += text.size().x;
            text_width = text.size().x;
            desired_size.y = desired_size.y.max(text.size().y);
        }
        if let Some(secondary_text) = &secondary_text {
            desired_size.x = (img_plus_spacing_width + text_width)
                .max(img_plus_spacing_width + secondary_text.size().x);
            if text.is_some() {
                desired_size.y = desired_size
                    .y
                    .max(desired_size.y + ui.spacing().item_spacing.y + secondary_text.size().y);
            } else {
                desired_size.y = desired_size.y.max(secondary_text.size().y);
            }
        }
        if let Some(shortcut_text) = &shortcut_text {
            desired_size.x += ui.spacing().item_spacing.x + shortcut_text.size().x;
            desired_size.y = desired_size.y.max(shortcut_text.size().y);
        }

        desired_size += 2.0 * button_padding;
        if !small {
            desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);
        }
        desired_size = desired_size.at_least(min_size);

        let (rect, mut response) = ui.allocate_at_least(desired_size, sense);
        response.widget_info(|| {
            if let Some(text) = &text {
                WidgetInfo::labeled(WidgetType::Button, true, text.text())
            } else {
                WidgetInfo::new(WidgetType::Button)
            }
        });

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            let (frame_expansion, frame_rounding, frame_fill, frame_stroke) = if selected {
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
            let frame_rounding = rounding.unwrap_or(frame_rounding);
            let frame_fill = fill.unwrap_or(frame_fill);
            let frame_stroke = stroke.unwrap_or(frame_stroke);
            ui.painter().rect(
                rect.expand2(frame_expansion),
                frame_rounding,
                frame_fill,
                frame_stroke,
                StrokeKind::Outside,
            );

            let mut cursor_x = rect.min.x + button_padding.x;

            if let Some(image) = &image {
                match image {
                    Composite::Image(image) => {
                        let image_rect = Rect::from_min_size(
                            pos2(cursor_x, rect.center().y - 0.5 - (image_size.y / 2.0)),
                            image_size,
                        );
                        cursor_x += image_size.x;
                        let tlr = image.load_for_size(ui.ctx(), image_size);
                        paint_texture_load_result(
                            ui,
                            &tlr,
                            image_rect,
                            show_loading_spinner,
                            image.image_options(),
                        );

                        response =
                            texture_load_result_response(&image.source(ui.ctx()), &tlr, response);
                    }
                    Composite::Icon(icon) => {
                        let galley = WidgetText::RichText(icon.clone().size(image_size.y))
                            .into_galley(
                                ui,
                                wrap.map(|wrap| {
                                    if wrap {
                                        TextWrapMode::Wrap
                                    } else {
                                        TextWrapMode::Extend
                                    }
                                }),
                                text_wrap_width,
                                TextStyle::Button,
                            );
                        let image_rect = Rect::from_min_size(
                            pos2(cursor_x, rect.center().y - 0.5 - (galley.size().y / 2.0)),
                            galley.size(),
                        );
                        cursor_x += galley.size().x;
                        // galley.paint_with_fallback_color(
                        //     ui.painter(),
                        //     image_rect.min,
                        //     visuals.fg_stroke.color,
                        // );

                        ui.painter()
                            .galley(image_rect.min, galley, visuals.fg_stroke.color);
                    }
                }
            }

            if image.is_some() && (text.is_some() || secondary_text.is_some()) {
                cursor_x += ui.spacing().icon_spacing;
            }

            let mut text_max_y = None;
            if let Some(text) = text {
                let text_pos =
                    if image.is_some() || shortcut_text.is_some() || secondary_text.is_some() {
                        if secondary_text.is_some() {
                            let h = text.size().y + ui.spacing().item_spacing.y;
                            let height = h + secondary_text.as_ref().unwrap().size().y;
                            let y = rect.center().y - 0.5 * height;
                            text_max_y = Some(y + h);
                            pos2(cursor_x, y)
                        } else {
                            pos2(cursor_x, rect.center().y - 0.5 * text.size().y)
                        }
                    } else {
                        // Make sure button text is centered if within a centered layout
                        ui.layout()
                            .align_size_within_rect(text.size(), rect.shrink2(button_padding))
                            .min
                    };
                //text.paint_with_visuals(ui.painter(), text_pos, visuals);
                ui.painter().galley(text_pos, text, visuals.text_color());
            }

            if let Some(secondary_text) = secondary_text {
                let y =
                    text_max_y.unwrap_or_else(|| rect.center().y - 0.5 * secondary_text.size().y);
                let secondary_text_pos = pos2(cursor_x, y);
                //secondary_text.paint_with_visuals(ui.painter(), secondary_text_pos, visuals);
                ui.painter()
                    .galley(secondary_text_pos, secondary_text, visuals.text_color());
            }

            if let Some(shortcut_text) = shortcut_text {
                let shortcut_text_pos = pos2(
                    rect.max.x - button_padding.x - shortcut_text.size().x,
                    rect.center().y - 0.5 * shortcut_text.size().y,
                );
                // shortcut_text.paint_with_fallback_color(
                //     ui.painter(),
                //     shortcut_text_pos,
                //     ui.visuals().weak_text_color(),
                // );
                ui.painter()
                    .galley(shortcut_text_pos, shortcut_text, visuals.text_color());
            }

            if pulldown_selector {
                let galley = WidgetText::RichText(RichText::new("⏷").size(14.)).into_galley(
                    ui,
                    wrap.map(|wrap| {
                        if wrap {
                            TextWrapMode::Wrap
                        } else {
                            TextWrapMode::Extend
                        }
                    }),
                    text_wrap_width,
                    TextStyle::Button,
                );
                let image_rect = Rect::from_min_size(
                    pos2(
                        rect.max.x - button_padding.x - 20.0,
                        rect.center().y - 0.5 - (galley.size().y / 2.0),
                    ),
                    galley.size(),
                );

                // galley.paint_with_fallback_color(
                //     ui.painter(),
                //     image_rect.min,
                //     visuals.fg_stroke.color,
                // );
                ui.painter()
                    .galley(image_rect.min, galley, visuals.fg_stroke.color);
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

fn font_height(text: &WidgetText, fonts: &epaint::Fonts, style: &Style) -> f32 {
    match text {
        WidgetText::RichText(text) => text.font_height(fonts, style),
        WidgetText::LayoutJob(job) => job.font_height(fonts),
        WidgetText::Galley(galley) => {
            if let Some(row) = galley.rows.first() {
                row.height()
            } else {
                galley.size().y
            }
        }
    }
}

pub fn texture_load_result_response(
    source: &ImageSource<'_>,
    tlr: &TextureLoadResult,
    response: Response,
) -> Response {
    match tlr {
        Ok(TexturePoll::Ready { .. }) => response,
        Ok(TexturePoll::Pending { .. }) => {
            let uri = source.uri().unwrap_or("image");
            response.on_hover_text(format!("Loading {uri}…"))
        }
        Err(err) => {
            let uri = source.uri().unwrap_or("image");
            response.on_hover_text(format!("Failed loading {uri}: {err}"))
        }
    }
}

pub fn paint_texture_load_result(
    ui: &Ui,
    tlr: &TextureLoadResult,
    rect: Rect,
    show_loading_spinner: Option<bool>,
    options: &ImageOptions,
) {
    match tlr {
        Ok(TexturePoll::Ready { texture }) => {
            paint_texture_at(ui.painter(), rect, options, texture);
        }
        Ok(TexturePoll::Pending { .. }) => {
            let show_loading_spinner =
                show_loading_spinner.unwrap_or(ui.visuals().image_loading_spinners);
            if show_loading_spinner {
                Spinner::new().paint_at(ui, rect);
            }
        }
        Err(_) => {
            let font_id = TextStyle::Body.resolve(ui.style());
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "⚠",
                font_id,
                ui.visuals().error_fg_color,
            );
        }
    }
}
