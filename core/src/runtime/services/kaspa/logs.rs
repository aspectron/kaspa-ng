use crate::imports::*;

pub enum Log {
    Debug(String),
    Trace(String),
    Info(String),
    Warning(String),
    Error(String),
    Processed(String),
}

impl From<&str> for Log {
    fn from(line: &str) -> Self {
        let line = line.trim();
        if !line.is_empty() {
            if line.len() < 38 || &line[30..31] != "[" {
                Log::Info(line.to_string())
            } else {
                let time = &line[11..23];
                let kind = &line[31..36];
                let text = &line[38..];
                // text
                match kind {
                    "WARN " => Log::Warning(format!("{time} {text}")),
                    "ERROR" => Log::Error(format!("{time} {text}")),
                    _ => {
                        if text.starts_with("Processed") {
                            Log::Processed(format!("{time} {text}"))
                        } else {
                            Log::Info(format!("{time} {text}"))
                        }
                    }
                }
            }
        } else {
            Log::Info(line.to_string())
        }
    }
}

impl From<&Log> for RichText {
    fn from(log: &Log) -> Self {
        let text = match log {
            Log::Info(text) => RichText::from(text).color(theme_color().logs_info_color),
            Log::Error(text) => RichText::from(text).color(theme_color().logs_error_color),
            Log::Warning(text) => RichText::from(text).color(theme_color().logs_warning_color),
            Log::Debug(text) => RichText::from(text).color(theme_color().logs_debug_color),
            Log::Trace(text) => RichText::from(text).color(theme_color().logs_trace_color),
            Log::Processed(text) => RichText::from(text).color(theme_color().logs_processed_color),
        };

        text.font(FontId::monospace(theme_style().node_log_font_size))
    }
}

impl std::fmt::Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Log::Info(text) => text.to_owned(),
            Log::Error(text) => text.to_owned(),
            Log::Warning(text) => text.to_owned(),
            Log::Debug(text) => text.to_owned(),
            Log::Trace(text) => text.to_owned(),
            Log::Processed(text) => text.to_owned(),
        };

        write!(f, "{}", text)
    }
}
