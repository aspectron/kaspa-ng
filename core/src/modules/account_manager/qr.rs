use crate::imports::*;
use super::*;

pub struct Qr { }

impl Qr {
    pub fn render(ui : &mut Ui, rc : &RenderContext) {
        let RenderContext { context, .. } = rc;

        ui.add(
            Image::new(ImageSource::Bytes { uri : Cow::Owned(context.uri()), bytes: context.qr() })
            .fit_to_original_size(1.0)
            .texture_options(TextureOptions::NEAREST)
        );
    }
}