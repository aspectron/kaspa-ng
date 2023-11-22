use egui::Color32;

pub trait Color32Extension {
    fn from_f32(value: f32) -> Self;
}

impl Color32Extension for Color32 {
    fn from_f32(value: f32) -> Self {
        Color32::from_rgba_premultiplied(
            (value * 255.0) as u8,
            (value * 255.0) as u8,
            (value * 255.0) as u8,
            (value * 255.0) as u8,
        )
    }
}
