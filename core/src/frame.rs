use crate::imports::*;

pub fn window_frame(
    enable: bool,
    ctx: &egui::Context,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    let (is_fullscreen, is_maximized) = ctx.input(|i| {
        let viewport = i.viewport();
        (
            viewport.fullscreen.unwrap_or(false),
            viewport.maximized.unwrap_or(false),
        )
    });

    cfg_if! {
        if #[cfg(target_os = "macos")] {
            let hide = is_fullscreen;
        } else {
            let hide = false;
        }
    }

    if enable && !hide {
        let mut stroke = ctx.style().visuals.widgets.noninteractive.fg_stroke;

        let (rounding, stroke_width) = if is_fullscreen || is_maximized {
            (0.0.into(), 0.0)
        } else {
            (10.0.into(), 1.0)
        };

        stroke.width = stroke_width;

        let panel_frame = egui::Frame {
            fill: ctx.style().visuals.window_fill(),
            rounding,
            stroke,
            // stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
            outer_margin: 0.5.into(), // so the stroke is within the bounds
            ..Default::default()
        };

        let outline_frame = egui::Frame {
            // fill: ctx.style().visuals.window_fill(),
            rounding,
            stroke,
            // stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
            outer_margin: 0.5.into(), // so the stroke is within the bounds
            ..Default::default()
        };

        CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            let app_rect = ui.max_rect();

            let title_bar_height = 28.0;
            let title_bar_rect = {
                let mut rect = app_rect;
                rect.max.y = rect.min.y + title_bar_height;
                rect
            };
            title_bar_ui(ui, title_bar_rect, title, is_fullscreen, is_maximized);

            // Add the contents:
            let content_rect = {
                let mut rect = app_rect;
                rect.min.y = title_bar_rect.max.y;
                rect
            };
            // .shrink(4.0);
            // .shrink2(vec2(8.0,4.0));
            let mut content_ui = ui.child_ui(content_rect, *ui.layout());
            add_contents(&mut content_ui);

            // panel_frame.show(ui);
            ui.painter().add(outline_frame.paint(app_rect));
        });
    } else {
        let panel_frame = egui::Frame {
            fill: ctx.style().visuals.window_fill(),
            inner_margin: 0.0.into(),
            outer_margin: 0.0.into(),
            ..Default::default()
        };

        CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            let app_rect = ui.max_rect();
            let mut content_ui = ui.child_ui(app_rect, *ui.layout());
            add_contents(&mut content_ui);
        });
    }
}

fn title_bar_ui(
    ui: &mut egui::Ui,
    title_bar_rect: eframe::epaint::Rect,
    title: &str,
    is_fullscreen: bool,
    is_maximized: bool,
) {
    use egui::*;

    let painter = ui.painter();

    let title_bar_response = ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click());

    // Paint the title:
    painter.text(
        title_bar_rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(16.0),
        ui.style().visuals.text_color(),
    );

    // Paint the line under the title:
    painter.line_segment(
        [
            title_bar_rect.left_bottom() + vec2(1.0, 0.0),
            title_bar_rect.right_bottom() + vec2(-1.0, 0.0),
        ],
        ui.visuals().widgets.noninteractive.bg_stroke,
    );

    // Interact with the title bar (drag to move window):
    if title_bar_response.double_clicked() {
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx()
            .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
    } else if title_bar_response.is_pointer_button_down_on() {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }

    ui.allocate_ui_at_rect(title_bar_rect, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);
            close_maximize_minimize(ui, is_fullscreen, is_maximized);
        });
    });
}

/// Show some close/maximize/minimize buttons for the native window.
fn close_maximize_minimize(ui: &mut egui::Ui, is_fullscreen: bool, is_maximized: bool) {
    use egui_phosphor::light::*;

    let spacing = 8.0;
    let button_height = 16.0;

    let close_response = ui
        .add(Button::new(
            RichText::new(X.to_string()).size(button_height),
        ))
        // .add(Button::new(RichText::new("❌").size(button_height)))
        .on_hover_text(i18n("Close the window"));
    if close_response.clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }

    cfg_if! {
        if #[cfg(target_os = "macos")] {
            let support_fullscreen = true;
            let support_maximize = true;
        } else {
            let support_fullscreen = true;
            let support_maximize = true;
        }
    }

    if support_fullscreen && !is_maximized {
        ui.add_space(spacing);

        let is_fullscreen = ui.input(|i| i.viewport().fullscreen.unwrap_or(false));
        if is_fullscreen {
            let fullscreen_response = ui
                // .add(Button::new(RichText::new("🗗").size(button_height)))
                .add(Button::new(
                    RichText::new(ARROWS_IN.to_string()).size(button_height),
                ))
                .on_hover_text(i18n("Exit Full Screen"));
            if fullscreen_response.clicked() {
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::Fullscreen(false));
            }
        } else {
            let fullscreen_response = ui
                // .add(Button::new(RichText::new("🗗").size(button_height)))
                .add(Button::new(
                    RichText::new(ARROWS_OUT.to_string()).size(button_height),
                ))
                // .add(Button::new(RichText::new(ARROWS_OUT.to_string()).size(button_height)))
                .on_hover_text(i18n("Full Screen"));
            if fullscreen_response.clicked() {
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::Fullscreen(true));
            }
        }
    }

    if support_maximize && !is_fullscreen {
        ui.add_space(spacing);

        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        if is_maximized {
            let maximized_response = ui
                // .add(Button::new(RichText::new("🗗").size(button_height)))
                .add(Button::new(
                    RichText::new(RECTANGLE.to_string()).size(button_height),
                ))
                .on_hover_text(i18n("Restore window"));
            if maximized_response.clicked() {
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::Maximized(false));
            }
        } else {
            let maximized_response = ui
                // .add(Button::new(RichText::new("🗗").size(button_height)))
                .add(Button::new(
                    RichText::new(SQUARE.to_string()).size(button_height),
                ))
                // .add(Button::new(RichText::new(ARROWS_OUT.to_string()).size(button_height)))
                .on_hover_text(i18n("Maximize window"));
            if maximized_response.clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(true));
            }
        }
    }

    ui.add_space(spacing + 2.0);

    let minimized_response = ui
        .add(Button::new(RichText::new("🗕").size(button_height)))
        // .add(Button::new(RichText::new(ARROW_SQUARE_DOWN.to_string()).size(button_height)))
        // .add(Button::new(RichText::new(ARROW_LINE_DOWN.to_string()).size(button_height)))
        .on_hover_text(i18n("Minimize the window"));
    if minimized_response.clicked() {
        ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
    }
}
