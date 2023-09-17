use crate::imports::*;
use qrcode::render::svg;
use qrcode::*;

pub fn render_qrcode(text: &str, width: usize, height: usize) -> String {
    let code = QrCode::with_version(text, Version::Normal(4), EcLevel::L).unwrap();

    let _theme = crate::theme::theme();

    code.render::<svg::Color<'_>>()
        .min_dimensions(width as u32, height as u32)
        .dark_color(svg::Color("#ffffff"))
        .light_color(svg::Color("#00000000"))
        .build()
        .to_string()
}
// #[macro_use]
#[macro_export]
macro_rules! spawn {
    ($args: expr) => {{
        let id = concat!(file!(), ":", line!());
        println!("#### spawn ID: {}", id);
        let payload = Payload::new(id);
        if !payload.is_pending() {
            println!("spawning...");
            spawn_with_result(&payload, $args);
        }
        payload.take()
    }};
}

pub use spawn;
