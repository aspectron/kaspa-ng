pub use egui::SizeHint;
pub use egui_extras::image::{load_svg_bytes, load_svg_bytes_with_size};

pub fn color_image_to_icon_data(image: epaint::ColorImage) -> egui::IconData {
    egui::IconData {
        width: image.size[0] as u32,
        height: image.size[1] as u32,
        rgba: image.as_raw().to_vec(),
    }
}

pub fn svg_to_icon_data(svg_bytes: &[u8], size_hint: Option<SizeHint>) -> egui::IconData {
    let image = load_svg_bytes_with_size(svg_bytes, size_hint).unwrap();
    color_image_to_icon_data(image)
}
