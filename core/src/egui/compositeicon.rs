use egui::load::{TextureLoadResult, TexturePoll};
use egui::*;
use egui::widget_text::WidgetTextGalley;
use workflow_log::log_info;

/// Clickable button with text.
///
/// See also [`Ui::button`].
///
/// ```
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
pub struct CompositeIcon{
    icon: WidgetText,
    text: Option<WidgetText>,
    secondary_text: Option<WidgetText>,

    /// None means default for interact
    fill: Option<Color32>,
    stroke: Option<Stroke>,
    sense: Sense,
    small: bool,
    frame: Option<bool>,
    min_size: Vec2,
    rounding: Option<Rounding>,
    padding: Option<Vec2>,
    selected: bool,
    show_loading_spinner: Option<bool>,
}

impl CompositeIcon {
    pub fn new(icon:impl Into<WidgetText>) -> Self {
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
        icon: impl Into<WidgetText>,
        text: Option<impl Into<WidgetText>>,
        secondary_text: Option<impl Into<WidgetText>>,
    ) -> Self {
        //let icon = icon.map(|icon|egui::RichText::new(&icon).size(10.0).color(Color32::from_rgb(255, 0, 0)));
        Self {
            text: text.map(|a|a.into()),
            secondary_text: secondary_text.map(|a|a.into()),
            icon:icon.into(),
            fill: None,
            stroke: None,
            sense: Sense::click(),
            small: false,
            frame: None,
            min_size: Vec2::ZERO,
            rounding: None,
            padding: None,
            selected: false,
            show_loading_spinner: None,
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

    /// Set the minimum size of the button.
    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    /// Set the rounding of the button.
    pub fn rounding(mut self, rounding: impl Into<Rounding>) -> Self {
        self.rounding = Some(rounding.into());
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

    /// Do layout and position the galley in the ui, without painting it or adding widget info.
    pub fn layout_in_ui(&self, ui: &mut Ui) -> (Pos2, WidgetTextGalley, Response) {
        let sense = {
            // We only want to focus labels if the screen reader is on.
            if ui.memory(|mem| mem.options.screen_reader) {
                Sense::focusable_noninteractive()
            } else {
                Sense::click()
            }
        };

        let frame = self.frame.unwrap_or_else(|| ui.visuals().button_frame);

        let mut button_padding = self.padding.unwrap_or(if frame {
            ui.spacing().button_padding
        } else {
            Vec2::ZERO
        });
        if self.small {
            button_padding.y = 0.0;
        }

        let mut text_size = Vec2::ZERO;
        let mut text_wrap_width = ui.available_width() - 2.0 * button_padding.x;
        let mut secondary_text_style = TextStyle::Name("CompositeButtonSub".into());
        if !ui.style().text_styles.contains_key(&secondary_text_style) {
            secondary_text_style = TextStyle::Body;
        }

        let text = self.text.clone().map(|text| text.into_galley(ui, Some(true), text_wrap_width, TextStyle::Button));
        let secondary_text = self.secondary_text.clone()
            .map(|text| text.into_galley(ui, Some(true), text_wrap_width, secondary_text_style));
        
        if let Some(text) = text{
            text_size += text.size();
            text_size.y += ui.spacing().item_spacing.y;
        }
        if let Some(secondary_text) = secondary_text{
            text_size.y += ui.spacing().item_spacing.y + secondary_text.size().y;
            text_size.x += secondary_text.size().x.max(text_size.x);
        }

        if let WidgetText::Galley(galley) = self.icon.clone() {
            // If the user said "use this specific galley", then just use it:
            let mut size = galley.size();
            size.x = text_size.x.max(size.x);
            size.y += text_size.y;

            let (rect, response) = ui.allocate_exact_size(size, sense);
            let pos = match galley.job.halign {
                Align::LEFT => rect.left_top(),
                Align::Center => rect.center_top(),
                Align::RIGHT => rect.right_top(),
            };
            let text_galley = WidgetTextGalley {
                galley,
                galley_has_color: true,
            };
            return (pos, text_galley, response);
        }

        let valign = ui.layout().vertical_align();
        let mut text_job = self
            .icon.clone()
            .into_text_job(ui.style(), FontSelection::Default, valign);

        let truncate = true;//self.truncate;
        let wrap = !truncate && ui.wrap_text();//None.unwrap_or_else(|| ui.wrap_text());
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
            egui_assert!(first_row_indentation.is_finite());

            text_job.job.wrap.max_width = available_width;
            text_job.job.first_row_min_height = cursor.height();
            text_job.job.halign = Align::Min;
            text_job.job.justify = false;
            if let Some(first_section) = text_job.job.sections.first_mut() {
                first_section.leading_space = first_row_indentation;
            }
            let text_galley = ui.fonts(|f| text_job.into_galley(f));

            let pos = pos2(ui.max_rect().left(), ui.cursor().top());
            assert!(
                !text_galley.galley.rows.is_empty(),
                "Galleys are never empty"
            );
            // collect a response from many rows:
            let mut rect = text_galley.galley.rows[0]
                .rect;
            let mut rect_size = rect.size();
            rect_size.y += text_size.y;
            rect_size.x = text_size.x.max(rect_size.x);
            rect.set_width(rect_size.x);
            rect.set_height(rect_size.y);

            let rect = rect.translate(vec2(pos.x, pos.y));
            let mut response = ui.allocate_rect(rect, sense);
            for row in text_galley.galley.rows.iter().skip(1) {
                let rect = row.rect.translate(vec2(pos.x, pos.y));
                response |= ui.allocate_rect(rect, sense);
            }
            (pos, text_galley, response)
        } else {
            if truncate {
                text_job.job.wrap.max_width = available_width;
                text_job.job.wrap.max_rows = 1;
                text_job.job.wrap.break_anywhere = true;
            } else if wrap {
                text_job.job.wrap.max_width = available_width;
            } else {
                text_job.job.wrap.max_width = f32::INFINITY;
            };

            // if ui.is_grid() {
            //     // TODO(emilk): remove special Grid hacks like these
            //     text_job.job.halign = Align::LEFT;
            //     text_job.job.justify = false;
            // } else {
                text_job.job.halign = ui.layout().horizontal_placement();
                text_job.job.justify = ui.layout().horizontal_justify();
            //};

            let text_galley = ui.fonts(|f| text_job.into_galley(f));
            
            let mut size = text_galley.size();
            size.y += text_size.y;
            size.x = text_size.x.max(size.x);

            let (rect, response) = ui.allocate_exact_size(size, sense);
            let pos = match text_galley.galley.job.halign {
                Align::LEFT => rect.left_top(),
                Align::Center => rect.center_top(),
                Align::RIGHT => rect.right_top(),
            };
            (pos, text_galley, response)
        }
    }
}

impl Widget for CompositeIcon {
    fn ui(self, ui: &mut Ui) -> Response {

        let (pos, text_galley, mut response) = self.layout_in_ui(ui);

        let Self {
            text,
            icon,
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
        } = self;

        
        
        // if text_galley.galley.elided {
        //     // Show the full (non-elided) text on hover:
        //     response = response.on_hover_text(text_galley.text());
        // }

        // if ui.is_rect_visible(response.rect) {
        //     let response_color = ui.style().interact(&response).text_color();

        //     let underline = if response.has_focus() || response.highlighted() {
        //         Stroke::new(1.0, response_color)
        //     } else {
        //         Stroke::NONE
        //     };

        //     let override_text_color = if text_galley.galley_has_color {
        //         None
        //     } else {
        //         Some(response_color)
        //     };

        //     ui.painter().add(epaint::TextShape {
        //         pos,
        //         galley: text_galley.galley,
        //         override_text_color,
        //         underline,
        //         angle: 0.0,
        //     });
        // }

        // response
        
        
        
        // let space_available_for_image = if let Some(text) = &text {
        //     let font_height = ui.fonts(|fonts| font_height(text, fonts, ui.style()));
        //     Vec2::splat(font_height) // Reasonable?
        // } else {
        //     ui.available_size() - 2.0 * button_padding
        // };

        // let image_size = response.rect.size();

        
        // let mut desired_size = Vec2::ZERO;
        // let mut img_plus_spacing_height = 0.0;
        // //if icon.is_some() {
        //     desired_size.y += image_size.y;
        //     //img_plus_spacing_height += image_size.y;
        //     desired_size.y = desired_size.y.max(image_size.y);

        //     // if text.is_some() || secondary_text.is_some() {
        //     //     desired_size.y += ui.spacing().icon_spacing;
        //     //     //img_plus_spacing_height += ui.spacing().icon_spacing;
        //     // }
        // //}
        // //let mut text_height = 0.0;
        // if let Some(text) = &text {
        //     //desired_size.y += text.size().y;
        //     //text_height = text.size().y;
        //     desired_size.y += ui.spacing().icon_spacing + text.size().y;
        // }
        // if let Some(secondary_text) = &secondary_text {
        //     // desired_size.y = (img_plus_spacing_height + text_height)
        //     //     .max(img_plus_spacing_height + secondary_text.size().y);
        //     // if text.is_some() {
        //     //     desired_size.y = desired_size
        //     //         .y
        //     //         .max(desired_size.y + ui.spacing().item_spacing.y + secondary_text.size().y);
        //     // } else {
        //     //     desired_size.y = desired_size.y.max(secondary_text.size().y);
        //     // }
        //     desired_size.y += ui.spacing().item_spacing.y + secondary_text.size().y;
        // }
        

        // desired_size += 2.0 * button_padding;
        // if !self.small {
        //     desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);
        // }
        // desired_size = desired_size.at_least(min_size);

        //let rect = Rect::from([pos2(198.0, 0.0), pos2(280.0, 90.0)]);

        //response |= ui.allocate_rect(rect, sense);
        //let response = response.clone() | response.clone().with_new_rect(rect);

        //let (rect2, mut response2) = ui.allocate_ui_at_rect(desired_size, sense);
        //log_info!("rect {:?}", rect);
        //log_info!("response.rect {:?}", response.rect);
        //log_info!("rect2 {:?}", rect2);
        //response |= response2;
        response.widget_info(|| {
            if let Some(text) = &text {
                WidgetInfo::labeled(WidgetType::Button, text.text())
            } else {
                WidgetInfo::new(WidgetType::Button)
            }
        });

        let rect = response.rect;

        if ui.is_rect_visible(response.rect) {
            let frame = self.frame.unwrap_or_else(|| ui.visuals().button_frame);
            let visuals = ui.style().interact(&response);

            let (frame_expansion, frame_rounding, frame_fill, frame_stroke) = if selected {
                let selection = ui.visuals().selection;
                (
                    Vec2::ZERO,
                    Rounding::ZERO,
                    selection.bg_fill,
                    selection.stroke,
                )
            } else if frame {
                let expansion = Vec2::splat(visuals.expansion);
                (
                    expansion,
                    visuals.rounding,
                    visuals.weak_bg_fill, //Color32::RED,
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
            );

            //let mut cursor_x = rect.min.x + button_padding.x;

            //if let Some(icon) = &icon {
                // let image_rect = Rect::from_min_size(
                //     pos2(cursor_x, rect.center().y - 0.5 - (image_size.y / 2.0)),
                //     image_size,
                // );
                //cursor_x += image_size.x;
                //let tlr = image.load_for_size(ui.ctx(), image_size);
                // paint_texture_load_result(
                //     ui,
                //     &tlr,
                //     image_rect,
                //     show_loading_spinner,
                //     image.image_options(),
                // );

                //response = texture_load_result_response(image.source(), &tlr, response);
            //}
            let override_text_color = if text_galley.galley_has_color {
                None
            } else {
                Some(ui.style().interact(&response).text_color())
            };

            ui.painter().add(epaint::TextShape {
                pos,
                galley: text_galley.galley,
                override_text_color,
                underline: Stroke::NONE,
                angle: 0.0,
            });

            // if text.is_some() || secondary_text.is_some() {
            //     cursor_x += ui.spacing().icon_spacing;
            // }

            // let mut text_max_y = None;
            // if let Some(text) = text {
            //     let text_pos =
            //         if secondary_text.is_some() {
            //             if secondary_text.is_some() {
            //                 let h = text.size().y + ui.spacing().item_spacing.y;
            //                 let height = h + secondary_text.as_ref().unwrap().size().y;
            //                 let y = rect.center().y - 0.5 * height;
            //                 text_max_y = Some(y + h);
            //                 pos2(cursor_x, y)
            //             } else {
            //                 pos2(cursor_x, rect.center().y - 0.5 * text.size().y)
            //             }
            //         } else {
            //             // Make sure button text is centered if within a centered layout
            //             ui.layout()
            //                 .align_size_within_rect(text.size(), rect.shrink2(button_padding))
            //                 .min
            //         };
            //     text.paint_with_visuals(ui.painter(), text_pos, visuals);
            // }

            // if let Some(secondary_text) = secondary_text {
            //     let y =
            //         text_max_y.unwrap_or_else(|| rect.center().y - 0.5 * secondary_text.size().y);
            //     let secondary_text_pos = pos2(cursor_x, y);
            //     secondary_text.paint_with_visuals(ui.painter(), secondary_text_pos, visuals);
            // }

            // if let Some(shortcut_text) = shortcut_text {
            //     let shortcut_text_pos = pos2(
            //         rect.max.x - button_padding.x - shortcut_text.size().x,
            //         rect.center().y - 0.5 * shortcut_text.size().y,
            //     );
            //     shortcut_text.paint_with_fallback_color(
            //         ui.painter(),
            //         shortcut_text_pos,
            //         ui.visuals().weak_text_color(),
            //     );
            // }
        }

        if let Some(cursor) = ui.visuals().interact_cursor {
            if response.hovered {
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
