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
    fn large_button_enabled(&mut self, enabled: bool, text: impl Into<WidgetText>) -> Response;
    fn confirm_medium(
        &mut self,
        align: Align,
        ack: impl Into<WidgetText>,
        nack: impl Into<WidgetText>,
    ) -> Option<Confirm>;
    fn confirm_medium_apply_cancel(&mut self, align: Align) -> Option<Confirm>;
}

impl UiExtension for Ui {
    fn medium_button_enabled(&mut self, enabled: bool, text: impl Into<WidgetText>) -> Response {
        self.add_enabled(
            enabled,
            Button::new(text).min_size(theme_style().medium_button_size()),
        )
    }

    fn large_button_enabled(&mut self, enabled: bool, text: impl Into<WidgetText>) -> Response {
        self.add_enabled(
            enabled,
            Button::new(text).min_size(theme_style().large_button_size()),
        )
    }

    fn confirm_medium(
        &mut self,
        align: Align,
        ack: impl Into<WidgetText>,
        nack: impl Into<WidgetText>,
    ) -> Option<Confirm> {
        let mut resp: Option<Confirm> = None;
        self.horizontal(|ui| {
            if matches!(align, Align::Max) {
                ui.add_space(
                    ui.available_width()
                        - 16.
                        - (theme_style().medium_button_size.x + ui.spacing().item_spacing.x) * 2.,
                );
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

    fn confirm_medium_apply_cancel(&mut self, align: Align) -> Option<Confirm> {
        let _theme = theme();

        self.confirm_medium(
            align,
            format!("{} {}", egui_phosphor::light::CHECK, "Apply"),
            format!("{} {}", egui_phosphor::light::X, "Cancel"),
        )
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
}

impl LayoutJobBuilder {
    pub fn new(width: f32, leading: f32, font_id: Option<FontId>) -> Self {
        let job = LayoutJob {
            wrap: TextWrapping {
                max_width: width,
                max_rows: 1,
                break_anywhere: true,
                overflow_character: Some('…'),
            },
            ..Default::default()
        };

        Self {
            job,
            leading,
            font_id,
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
            self.leading,
            TextFormat {
                color,
                font_id: self.icon_font_id.clone().unwrap_or_default(),
                ..Default::default()
            },
        );

        self
    }

    pub fn label(self, ui: &mut Ui) -> Response {
        ui.label(self.job)
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
