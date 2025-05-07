use crate::imports::*;
use egui::text::TextWrapping;
use std::fmt::Debug;

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
    fn large_selected_button(&mut self, selected: bool, text: impl Into<WidgetText>) -> Response {
        self.large_button_enabled_selected(true, selected, text)
    }
    fn large_button_enabled(&mut self, enabled: bool, text: impl Into<WidgetText>) -> Response;
    fn large_button_enabled_selected(
        &mut self,
        enabled: bool,
        selected: bool,
        text: impl Into<WidgetText>,
    ) -> Response;
    fn confirm_medium(
        &mut self,
        align: Align,
        ack: Option<impl Into<WidgetText>>,
        nack: impl Into<WidgetText>,
    ) -> Option<Confirm>;
    fn confirm_medium_apply_cancel(&mut self, align: Align) -> Option<Confirm>;
    fn confirm_medium_cancel(&mut self, align: Align) -> Option<Confirm>;
    fn sized_separator(&mut self, size: Vec2) -> Response;
    fn widgets_rounding(&self) -> CornerRadius;
    fn small_separator(&mut self) {
        self.add_separator(self.create_separator(None, 0.5, None));
    }
    fn medium_separator(&mut self) {
        self.add_separator(self.create_separator(None, 0.3, None));
    }
    fn large_separator(&mut self) {
        self.add_separator(self.create_separator(None, 0.1, None));
    }
    fn small_separator_with_direction_and_spacing(&mut self, spacing: f32, is_horizontal: bool) {
        self.add_separator(self.create_separator(Some(spacing), 0.5, Some(is_horizontal)));
    }
    fn create_separator(
        &self,
        spacing: Option<f32>,
        shrink: f32,
        is_horizontal: Option<bool>,
    ) -> Separator;
    fn add_separator(&mut self, separator: Separator);
}

impl UiExtension for Ui {
    fn create_separator(
        &self,
        spacing: Option<f32>,
        shrink: f32,
        is_horizontal: Option<bool>,
    ) -> Separator {
        let mut sep = Separator::default();
        if let Some(spacing) = spacing {
            sep = sep.spacing(spacing)
        }
        if let Some(is_horizontal) = is_horizontal {
            if is_horizontal {
                sep = sep.horizontal();
            } else {
                sep = sep.vertical();
            }
        }

        //let sep = is_horizontal.map_or(sep, |is_horizontal| if is_horizontal{sep.horizontal()}else{sep.vertical()});

        let is_horizontal_line =
            is_horizontal.unwrap_or_else(|| !self.layout().main_dir().is_horizontal());

        let available_space = self.available_size_before_wrap();

        //log_info!("spacing:{spacing:?}, is_horizontal: {is_horizontal:?}, is_horizontal_line:{is_horizontal_line}");

        let size = if is_horizontal_line {
            available_space.x
        } else {
            available_space.y
        };

        let shrink = (size * shrink) / 2.0;

        sep.shrink(shrink)
    }
    fn add_separator(&mut self, separator: Separator) {
        self.add(separator);
    }

    fn widgets_rounding(&self) -> CornerRadius {
        self.visuals().widgets.hovered.corner_radius
    }

    fn medium_button_enabled(&mut self, enabled: bool, text: impl Into<WidgetText>) -> Response {
        self.add_enabled(
            enabled,
            Button::new(text)
                .corner_radius(self.widgets_rounding())
                .min_size(theme_style().medium_button_size()),
        )
    }

    fn large_button_enabled(&mut self, enabled: bool, text: impl Into<WidgetText>) -> Response {
        self.add_enabled(
            enabled,
            Button::new(text)
                .corner_radius(self.widgets_rounding())
                .min_size(theme_style().large_button_size()),
        )
    }

    fn large_button_enabled_selected(
        &mut self,
        enabled: bool,
        selected: bool,
        text: impl Into<WidgetText>,
    ) -> Response {
        self.add_enabled(
            enabled,
            Button::new(text)
                .corner_radius(self.widgets_rounding())
                .selected(selected)
                .min_size(theme_style().large_button_size()),
        )
    }

    fn confirm_medium(
        &mut self,
        align: Align,
        ack: Option<impl Into<WidgetText>>,
        nack: impl Into<WidgetText>,
    ) -> Option<Confirm> {
        let mut resp: Option<Confirm> = None;
        self.horizontal(|ui| {
            let buttons = if ack.is_some() { 2. } else { 1. };

            if matches!(align, Align::Max) {
                ui.add_space(
                    ui.available_width()
                        - 16.
                        - (theme_style().medium_button_size.x + ui.spacing().item_spacing.x)
                            * buttons,
                );
            }

            if let Some(ack) = ack {
                if ui.medium_button(ack).clicked() {
                    resp.replace(Confirm::Ack);
                }
            }

            if ui.medium_button(nack).clicked() {
                resp.replace(Confirm::Nack);
            }
        });

        resp
    }

    fn confirm_medium_apply_cancel(&mut self, align: Align) -> Option<Confirm> {
        let _theme = theme();

        self.confirm_medium(
            align,
            Some(format!("{} {}", egui_phosphor::light::CHECK, i18n("Apply"))),
            format!("{} {}", egui_phosphor::light::X, i18n("Cancel")),
        )
    }

    fn confirm_medium_cancel(&mut self, align: Align) -> Option<Confirm> {
        let _theme = theme();

        self.confirm_medium(
            align,
            Option::<&str>::None,
            format!("{} {}", egui_phosphor::light::X, i18n("Cancel")),
        )
    }

    fn sized_separator(&mut self, size: Vec2) -> Response {
        self.add_sized(size, Separator::default())
    }
}

pub struct LayoutJobBuilderSettings {
    pub width: f32,
    pub leading: f32,
    pub font_id: Option<FontId>,
}

impl LayoutJobBuilderSettings {
    pub fn new(width: f32, leading: f32, font_id: Option<FontId>) -> Self {
        Self {
            width,
            leading,
            font_id,
        }
    }
}

pub fn ljb(settings: &LayoutJobBuilderSettings) -> LayoutJobBuilder {
    LayoutJobBuilder::new(settings.width, settings.leading, settings.font_id.clone())
}

pub fn ljb_with_settings(width: f32, leading: f32, font_id: &FontId) -> LayoutJobBuilder {
    LayoutJobBuilder::new(width, leading, Some(font_id.clone()))
}

#[derive(Default)]
pub struct LayoutJobBuilder {
    job: LayoutJob,
    leading: f32,
    icon_font_id: Option<FontId>,
    font_id: Option<FontId>,
    heading: Option<(f32, f32, String, Color32)>,
}

impl LayoutJobBuilder {
    pub fn new(width: f32, leading: f32, font_id: Option<FontId>) -> Self {
        let job = LayoutJob {
            wrap: TextWrapping {
                max_width: width,
                max_rows: 4,
                break_anywhere: true,
                overflow_character: Some('…'),
            },
            ..Default::default()
        };

        Self {
            job,
            leading,
            font_id,
            heading: None,
            ..Default::default()
        }
    }

    pub fn with_icon_font(mut self, icon_font_id: FontId) -> Self {
        self.icon_font_id = Some(icon_font_id);
        self
    }

    pub fn text(mut self, text: &str, color: Color32) -> Self {
        self.job.append(
            text,
            self.leading,
            TextFormat {
                color,
                font_id: self.font_id.clone().unwrap_or_default(),
                ..Default::default()
            },
        );

        self
    }

    pub fn padded(mut self, width: usize, text: &str, color: Color32) -> Self {
        self.job.append(
            text.pad_to_width_with_alignment(width, Alignment::Right)
                .as_str(),
            self.leading,
            TextFormat {
                color,
                font_id: self.font_id.clone().unwrap_or_default(),
                ..Default::default()
            },
        );

        self
    }
    pub fn icon(mut self, text: &str, color: Color32) -> Self {
        self.job.append(
            text,
            4.,
            TextFormat {
                color,
                font_id: self.icon_font_id.clone().unwrap_or_default(),
                ..Default::default()
            },
        );

        self
    }

    pub fn heading(mut self, ui: &mut Ui, width: f32, text: &str, color: Color32) -> Self {
        let galley = ui.painter().layout_no_wrap(
            text.to_string(),
            self.font_id.clone().unwrap_or_default(),
            color,
        );
        self.heading = Some((width, galley.size().y, text.to_string(), color));
        self
    }

    pub fn label(self, ui: &mut Ui) -> Response {
        Self::render_label(ui, self.job, self.heading)
    }

    fn render_label(
        ui: &mut Ui,
        job: LayoutJob,
        heading: Option<(f32, f32, String, Color32)>,
    ) -> Response {
        if let Some((x, y, text, color)) = heading {
            let desired_size = Vec2 { x, y };
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    desired_size,
                    Layout::right_to_left(Align::Center),
                    |ui| ui.label(RichText::new(text).color(color).font(FontId::default())),
                );
                ui.label(job)
            })
            .inner
        } else {
            ui.label(job)
        }
    }

    pub fn hyperlink_with_clipboard_icon(
        self,
        ui: &mut Ui,
        text: &str,
        url: &str,
        color: Color32,
        clipboard_text: Option<String>,
    ) {
        ui.horizontal(|ui| {
            Self::render_label(ui, self.job, self.heading);
            ui.hyperlink_to_tab(
                RichText::new(text)
                    .font(self.font_id.unwrap_or_default())
                    .color(color),
                url,
            );
            if let Some(text) = clipboard_text {
                Self::clipboard_icon(ui, text);
            }
        });
    }
    pub fn hyperlink(self, ui: &mut Ui, text: &str, url: &str, color: Color32) {
        self.hyperlink_with_clipboard_icon(ui, text, url, color, None)
    }
    pub fn transaction_id(
        self,
        ui: &mut Ui,
        txid: &str,
        url: &str,
        color: Color32,
        range: Option<usize>,
    ) {
        self.hyperlink_with_clipboard_icon(
            ui,
            &format_partial_string(txid, range),
            url,
            color,
            Some(txid.to_string()),
        )
    }
    pub fn script(self, ui: &mut Ui, script: &str, color: Color32, range: Option<usize>) {
        let this = self.text(&format_partial_string(script, range), color);
        ui.horizontal(|ui| {
            Self::render_label(ui, this.job, this.heading);
            Self::clipboard_icon(ui, script.to_string());
        });
    }
    pub fn with_clipboard_icon(self, ui: &mut Ui, text: &str) {
        ui.horizontal(|ui| {
            Self::render_label(ui, self.job, self.heading);
            Self::clipboard_icon(ui, text.to_string());
        });
    }
    pub fn address(
        self,
        ui: &mut Ui,
        address: &str,
        url: &str,
        color: Color32,
        range: Option<usize>,
    ) {
        self.hyperlink_with_clipboard_icon(
            ui,
            &format_address_string(address, range),
            url,
            color,
            Some(address.to_string()),
        )
    }

    pub fn clipboard_icon(ui: &mut Ui, text: String) {
        if ui
            .add(Label::new(egui_phosphor::light::CLIPBOARD_TEXT).sense(Sense::click()))
            .clicked()
        {
            //ui.output_mut(|o| o.copied_text = text);
            ui.ctx().copy_text(text);
            runtime().notify_clipboard(i18n("Copied to clipboard"));
        }
    }
}

impl From<LayoutJobBuilder> for LayoutJob {
    fn from(builder: LayoutJobBuilder) -> Self {
        builder.job
    }
}

impl From<LayoutJobBuilder> for WidgetText {
    fn from(builder: LayoutJobBuilder) -> Self {
        builder.job.into()
    }
}

pub trait HyperlinkExtension {
    fn hyperlink_to_tab(&mut self, text: impl Into<WidgetText>, url: impl Into<String>)
        -> Response;
    fn hyperlink_url_to_tab(&mut self, url: impl Into<String>) -> Response;
}

impl HyperlinkExtension for Ui {
    fn hyperlink_to_tab(
        &mut self,
        text: impl Into<WidgetText>,
        url: impl Into<String>,
    ) -> Response {
        let url = url.into();
        Hyperlink::from_label_and_url(text, url)
            .open_in_new_tab(true)
            .ui(self)
    }
    fn hyperlink_url_to_tab(&mut self, url: impl Into<String>) -> Response {
        let url = url.into();
        Hyperlink::from_label_and_url(url.clone(), url)
            .open_in_new_tab(true)
            .ui(self)
    }
}

type TextEditorCreateFn<'editor> = Box<dyn FnOnce(&mut Ui, &mut String) -> Response + 'editor>;
type TextEditorChangeFn<'editor> = Box<dyn FnOnce(&str) + 'editor>;
type TextEditorSubmitFn<'editor, Focus> = Box<dyn FnOnce(&str, &mut FocusManager<Focus>) + 'editor>;

#[derive(Default, Debug)]
pub struct FocusManager<Focus>
where
    Focus: PartialEq + Debug,
{
    focus: Option<Focus>,
}

impl<Focus> FocusManager<Focus>
where
    Focus: PartialEq + Debug,
{
    pub fn next(&mut self, focus: Focus) {
        self.focus.replace(focus);
    }

    pub fn matches(&self, focus: Focus) -> bool {
        self.focus == Some(focus)
    }

    pub fn clear(&mut self) {
        self.focus.take();
    }

    pub fn take(&mut self) -> Option<Focus> {
        self.focus.take()
    }
}

pub struct TextEditor<'editor, Focus>
where
    Focus: PartialEq + Copy + Debug,
{
    user_text: &'editor mut String,
    focus_manager: &'editor mut FocusManager<Focus>,
    focus_value: Focus,
    editor_create_fn: TextEditorCreateFn<'editor>,
    editor_change_fn: Option<TextEditorChangeFn<'editor>>,
    editor_submit_fn: Option<TextEditorSubmitFn<'editor, Focus>>,
}

impl<'editor, Focus> TextEditor<'editor, Focus>
where
    Focus: PartialEq + Copy + Debug,
{
    pub fn new(
        user_text: &'editor mut String,
        focus_manager: &'editor mut FocusManager<Focus>,
        focus_value: Focus,
        editor_create_fn: impl FnOnce(&mut Ui, &mut String) -> Response + 'editor,
    ) -> Self {
        Self {
            user_text,
            focus_manager,
            focus_value,
            editor_create_fn: Box::new(editor_create_fn),
            editor_change_fn: None,
            editor_submit_fn: None,
        }
    }

    pub fn change(mut self, change: impl FnOnce(&str) + 'editor) -> Self {
        self.editor_change_fn = Some(Box::new(change));
        self
    }

    pub fn submit(mut self, submit: impl FnOnce(&str, &mut FocusManager<Focus>) + 'editor) -> Self {
        self.editor_submit_fn = Some(Box::new(submit));
        self
    }

    pub fn build(self, ui: &mut Ui) -> Response {
        let TextEditor {
            user_text,
            focus_manager,
            focus_value,
            editor_create_fn,
            editor_change_fn,
            editor_submit_fn,
        } = self;

        let mut editor_text = user_text.clone();
        let response = editor_create_fn(ui, &mut editor_text);

        if focus_manager.matches(focus_value) && !response.has_focus() {
            focus_manager.clear();
            response.request_focus();
        }

        if *user_text != editor_text {
            *user_text = editor_text;
            if let Some(editor_change_fn) = editor_change_fn {
                editor_change_fn(user_text.as_str());
            }
        } else if response.text_edit_submit(ui) {
            *user_text = editor_text;
            if let Some(editor_submit_fn) = editor_submit_fn {
                editor_submit_fn(user_text.as_str(), focus_manager);
            }
        }

        response
    }
}

pub trait WidgetSpacerExtension {
    fn space(&mut self);
}

impl WidgetSpacerExtension for Ui {
    fn space(&mut self) {
        self.add_space(theme_style().widget_spacing);
    }
}

pub fn set_menu_style(style: &mut Style) {
    style.spacing.button_padding = vec2(2.0, 0.0);
    style.visuals.widgets.active.bg_stroke = Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
}
