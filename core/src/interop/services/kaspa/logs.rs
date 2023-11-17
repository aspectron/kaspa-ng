use egui::RichText;

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
            Log::Info(text) => RichText::from(text).color(egui::Color32::WHITE),
            Log::Error(text) => RichText::from(text).color(egui::Color32::LIGHT_RED),
            Log::Warning(text) => RichText::from(text).color(egui::Color32::LIGHT_YELLOW),
            Log::Debug(text) => RichText::from(text).color(egui::Color32::LIGHT_BLUE),
            Log::Trace(text) => RichText::from(text).color(egui::Color32::LIGHT_GRAY),
            Log::Processed(text) => RichText::from(text).color(egui::Color32::LIGHT_GREEN),
        };

        text.monospace()
    }
}
