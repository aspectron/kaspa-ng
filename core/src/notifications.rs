use crate::imports::*;
use egui_notify::Toasts;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserNotifyKind {
    Info,
    Success,
    Warning,
    Error,
}

impl std::fmt::Display for UserNotifyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserNotifyKind::Info => write!(f, "info"),
            UserNotifyKind::Success => write!(f, "success"),
            UserNotifyKind::Warning => write!(f, "warning"),
            UserNotifyKind::Error => write!(f, "error"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UserNotification {
    pub kind: UserNotifyKind,
    pub message: String,
    pub duration: Option<Duration>,
    pub progress: bool,
    pub closable: bool,
    pub toast: bool,
}

impl Default for UserNotification {
    fn default() -> Self {
        Self {
            kind: UserNotifyKind::Info,
            message: String::new(),
            duration: Some(Duration::from_millis(3500)),
            progress: true,
            closable: false,
            toast: false,
        }
    }
}

impl UserNotification {
    pub fn new(kind: UserNotifyKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            message: text.into(),
            ..Default::default()
        }
    }

    pub fn as_toast(mut self) -> Self {
        self.toast = true;
        self
    }

    pub fn is_toast(&self) -> bool {
        self.toast
    }

    pub fn info(text: impl Into<String>) -> Self {
        Self::new(UserNotifyKind::Info, text)
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self::new(UserNotifyKind::Warning, text)
    }

    pub fn error(text: impl Into<String>) -> Self {
        let text = text.into();
        // println!("error: {}", text);
        Self::new(UserNotifyKind::Error, text).duration(Duration::from_millis(5000))
    }

    pub fn success(text: impl Into<String>) -> Self {
        Self::new(UserNotifyKind::Success, text)
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn short(mut self) -> Self {
        self.duration = Some(Duration::from_millis(1500));
        self
    }

    pub fn toast(self, toasts: &mut Toasts) {
        match self.kind {
            UserNotifyKind::Info => {
                toasts
                    .info(self.message)
                    .duration(self.duration)
                    .show_progress_bar(self.progress)
                    .closable(self.closable);
            }
            UserNotifyKind::Success => {
                toasts
                    .success(self.message)
                    .duration(self.duration)
                    .show_progress_bar(self.progress)
                    .closable(self.closable);
            }
            UserNotifyKind::Warning => {
                toasts
                    .warning(self.message)
                    .duration(self.duration)
                    .show_progress_bar(self.progress)
                    .closable(self.closable);
            }
            UserNotifyKind::Error => {
                toasts
                    .error(self.message)
                    .duration(self.duration)
                    .show_progress_bar(self.progress)
                    .closable(self.closable);
            }
        }
    }

    pub fn icon(&self) -> RichText {
        use egui_phosphor::thin::*;

        match self.kind {
            UserNotifyKind::Info => RichText::new(INFO).color(info_color()),
            UserNotifyKind::Success => RichText::new(INFO).color(strong_color()),
            UserNotifyKind::Warning => RichText::new(WARNING).color(warning_color()),
            UserNotifyKind::Error => RichText::new(SEAL_WARNING).color(error_color()),
        }
    }

    pub fn text(&self) -> RichText {
        match self.kind {
            UserNotifyKind::Info => RichText::new(&self.message),
            UserNotifyKind::Success => RichText::new(&self.message),
            UserNotifyKind::Warning => RichText::new(&self.message).color(warning_color()),
            UserNotifyKind::Error => RichText::new(&self.message).color(error_color()),
        }
    }
}

#[derive(Default)]
pub struct Notifications {
    pub notifications: Vec<UserNotification>,
    pub last_notification: usize,
    pub errors: bool,
    pub warnings: bool,
    pub infos: bool,
}

impl Notifications {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.notifications.clear();
        self.errors = false;
        self.warnings = false;
        self.infos = false;
    }

    pub fn has_some(&self) -> bool {
        !self.notifications.is_empty()
    }

    pub fn push(&mut self, notification: UserNotification) {
        if notification.kind == UserNotifyKind::Error {
            self.errors = true;
        } else if notification.kind == UserNotifyKind::Warning {
            self.warnings = true;
        } else if notification.kind == UserNotifyKind::Info {
            self.infos = true;
        }

        self.notifications.push(notification);
    }

    pub fn render(&mut self, ui: &mut Ui, device: &Device) {
        use egui_phosphor::light::*;

        if self.notifications.len() != self.last_notification {
            let id = PopupPanel::id(ui, "notification_popup");
            ui.memory_mut(|mem| mem.open_popup(id));
            self.last_notification = self.notifications.len();
        }

        let icon = if self.errors {
            RichText::new(SEAL_WARNING).color(error_color())
        } else if self.warnings {
            RichText::new(WARNING).color(warning_color())
        } else if self.infos {
            RichText::new(INFO).color(info_color())
        } else {
            RichText::new(INFO)
        };

        let screen_rect = ui.ctx().screen_rect();
        let width = (screen_rect.width() / 4. * 3.).min(500.);
        let height = (screen_rect.height() / 4.).min(240.);

        PopupPanel::new(
            PopupPanel::id(ui, "notification_popup"),
            // |ui| ui.add(Label::new(icon.size(16.)).sense(Sense::click())),
            |ui| ui.add(Label::new(icon.size(device.top_icon_size())).sense(Sense::click())),
            |ui, close| {
                egui::ScrollArea::vertical()
                    .id_salt("notification_popup_scroll")
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for notification in self.notifications.iter() {
                                ui.horizontal(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(notification.icon().size(20.));
                                    });
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(notification.text().size(14.));
                                    });
                                });
                            }
                        });
                    });

                ui.separator();
                ui.add_space(4.);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.medium_button(i18n("Close")).clicked() {
                        *close = true;
                    }
                    if ui.medium_button(i18n("Clear")).clicked() {
                        self.clear();
                        *close = true;
                    }
                    if ui
                        .medium_button(format!("{CLIPBOARD} {}", i18n("Copy")))
                        .clicked()
                    {
                        let notifications = self
                            .notifications
                            .iter()
                            .map(|notification| {
                                let UserNotification { message, kind, .. } = notification;
                                format!("[{}] {}", kind.to_string().to_uppercase(), message)
                            })
                            .collect::<Vec<String>>()
                            .join("\n");
                        //ui.output_mut(|o| o.copy_text(notifications));
                        ui.ctx().copy_text(notifications);
                        runtime().notify_clipboard(i18n("Copied to clipboard"));
                        *close = true;
                    }
                });
            },
        )
        .with_min_width(width)
        .with_max_height(height)
        .with_caption(i18n("Notifications"))
        .with_close_on_interaction(false)
        .build(ui);
    }
}
