use crate::imports::*;
use passwords::analyzer;
use passwords::scorer;

pub fn secret_score(password: impl AsRef<str>) -> f64 {
    scorer::score(&analyzer::analyze(password))
}

pub fn secret_score_to_text(score: f64) -> String {
    if (0.0..=20.0).contains(&score) {
        String::from(i18n("Very dangerous (may be cracked within few seconds)"))
    } else if score > 20.0 && score <= 40.0 {
        String::from(i18n("Dangerous"))
    } else if score > 40.0 && score <= 60.0 {
        String::from(i18n("Very weak"))
    } else if score > 60.0 && score <= 80.0 {
        String::from(i18n("Weak"))
    } else if score > 80.0 && score <= 90.0 {
        String::from(i18n("Good"))
    } else if score > 90.0 && score <= 95.0 {
        String::from(i18n("Strong"))
    } else if score > 95.0 && score <= 99.0 {
        String::from(i18n("Very strong"))
    } else if score > 99.0 && score <= 100.0 {
        String::from(i18n("Invulnerable"))
    } else {
        String::from("Value is outside the defined range")
    }
}

pub fn render_secret_score_text(ui: &mut Ui, prefix: impl Into<String>, score: f64) {
    let text = format!("{}: {}", prefix.into(), secret_score_to_text(score));

    let color = if score < 80.0 {
        error_color()
    } else if score < 90.0 {
        warning_color()
    } else {
        theme_color().strong_color
    };

    ui.colored_label(color, text);
}
