use egui_notify::Toasts;
use std::time::Duration;

#[derive(Clone)]
pub enum UserNotifyKind {
    Info,
    Success,
    Warning,
    Error,
    Basic,
}

#[derive(Clone)]
pub struct UserNotification {
    pub kind: UserNotifyKind,
    pub message: String,
    pub duration: Option<Duration>,
    pub progress: bool,
    pub closable: bool,
}

impl Default for UserNotification {
    fn default() -> Self {
        Self {
            kind: UserNotifyKind::Info,
            message: String::new(),
            duration: Some(Duration::from_millis(3500)),
            progress: true,
            closable: false,
        }
    }
}

impl UserNotification {
    pub fn new(kind: UserNotifyKind, text: &str) -> Self {
        Self {
            kind,
            message: text.to_string(),
            ..Default::default()
        }
    }

    pub fn info(text: &str) -> Self {
        Self::new(UserNotifyKind::Info, text)
    }

    pub fn warning(text: &str) -> Self {
        Self::new(UserNotifyKind::Warning, text)
    }

    pub fn error(text: &str) -> Self {
        Self::new(UserNotifyKind::Error, text)
    }

    pub fn success(text: &str) -> Self {
        Self::new(UserNotifyKind::Success, text)
    }

    pub fn basic(text: &str) -> Self {
        Self::new(UserNotifyKind::Basic, text)
    }

    pub fn short(mut self) -> Self {
        self.duration = Some(Duration::from_millis(1500));
        self
    }

    pub fn render(self, toasts: &mut Toasts) {
        match self.kind {
            UserNotifyKind::Info => {
                toasts
                    .info(self.message)
                    .set_duration(self.duration)
                    .set_show_progress_bar(self.progress)
                    .set_closable(self.closable);
            }
            UserNotifyKind::Success => {
                toasts
                    .success(self.message)
                    .set_duration(self.duration)
                    .set_show_progress_bar(self.progress)
                    .set_closable(self.closable);
            }
            UserNotifyKind::Warning => {
                toasts
                    .warning(self.message)
                    .set_duration(self.duration)
                    .set_show_progress_bar(self.progress)
                    .set_closable(self.closable);
            }
            UserNotifyKind::Error => {
                toasts
                    .error(self.message)
                    .set_duration(self.duration)
                    .set_show_progress_bar(self.progress)
                    .set_closable(self.closable);
            }
            UserNotifyKind::Basic => {
                toasts
                    .basic(self.message)
                    .set_duration(self.duration)
                    .set_show_progress_bar(self.progress)
                    .set_closable(self.closable);
            }
        }
    }
}
