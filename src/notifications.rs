use egui_notify::Toasts;
use std::time::Duration;

#[derive(Clone)]
pub enum Notify {
    Info,
    Success,
    Warning,
    Error,
    Basic,
}

#[derive(Clone)]
pub struct Notification {
    pub kind: Notify,
    pub message: String,
    pub duration: Option<Duration>,
    pub progress: bool,
    pub closable: bool,
}

impl Default for Notification {
    fn default() -> Self {
        Self {
            kind: Notify::Info,
            message: String::new(),
            duration: Some(Duration::from_millis(3500)),
            progress: true,
            closable: false,
        }
    }
}

impl Notification {
    pub fn new(kind: Notify, text: &str) -> Self {
        Self {
            kind,
            message: text.to_string(),
            ..Default::default()
        }
    }

    pub fn info(text: &str) -> Self {
        Self::new(Notify::Info, text)
    }

    pub fn warning(text: &str) -> Self {
        Self::new(Notify::Warning, text)
    }

    pub fn error(text: &str) -> Self {
        Self::new(Notify::Error, text)
    }

    pub fn success(text: &str) -> Self {
        Self::new(Notify::Success, text)
    }

    pub fn basic(text: &str) -> Self {
        Self::new(Notify::Basic, text)
    }

    pub fn render(self, toasts: &mut Toasts) {
        match self.kind {
            Notify::Info => {
                toasts
                    .info(self.message)
                    .set_duration(self.duration)
                    .set_show_progress_bar(self.progress)
                    .set_closable(self.closable);
            }
            Notify::Success => {
                toasts
                    .success(self.message)
                    .set_duration(self.duration)
                    .set_show_progress_bar(self.progress)
                    .set_closable(self.closable);
            }
            Notify::Warning => {
                toasts
                    .warning(self.message)
                    .set_duration(self.duration)
                    .set_show_progress_bar(self.progress)
                    .set_closable(self.closable);
            }
            Notify::Error => {
                toasts
                    .error(self.message)
                    .set_duration(self.duration)
                    .set_show_progress_bar(self.progress)
                    .set_closable(self.closable);
            }
            Notify::Basic => {
                toasts
                    .basic(self.message)
                    .set_duration(self.duration)
                    .set_show_progress_bar(self.progress)
                    .set_closable(self.closable);
            }
        }
    }
}
