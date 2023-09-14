use egui_extras::{image, RetainedImage};
use qrcode::render::svg;
use qrcode::*;

pub fn render_qrcode(text: &str, width: usize, height: usize) -> RetainedImage {
    let code = QrCode::with_version(text, Version::Normal(4), EcLevel::L).unwrap();

    let _theme = crate::theme::theme();

    let image = code
        .render::<svg::Color<'_>>()
        .min_dimensions(width as u32, height as u32)
        .dark_color(svg::Color("#ffffff"))
        .light_color(svg::Color("#00000000"))
        .build();

    RetainedImage::from_svg_bytes_with_size(text, image.as_bytes(), image::FitTo::Original).unwrap()
}
