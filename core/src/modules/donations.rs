use std::{borrow::Cow, collections::hash_map::Entry};

use crate::imports::*;
pub struct Donations {
    qr : HashMap<String, (String,load::Bytes)>,
}

impl Donations {

    pub const ADDRESS: &'static str = "kaspatest:qqdr2mv4vkes6kvhgy8elsxhvzwde42629vnpcxe4f802346rnfkklrhz0x7x";

    pub fn new(_runtime: Runtime) -> Self {
        Self { 
            qr : Default::default(),
        }
    }

    fn qr(&mut self) -> (String,load::Bytes) {

        let (uri,qr) = match self.qr.entry(theme_color().name.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let uri = format!("bytes://{}-{}.svg", Self::ADDRESS, theme_color().name);
                let qr = render_qrcode(&Self::ADDRESS, 128, 128);
                entry.insert((uri, qr.as_bytes().to_vec().into()))
            },
        };

        (uri.clone(),qr.clone())
    }

}

impl ModuleT for Donations {

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Mobile
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        let (uri, qr) = self.qr();
        let back = Rc::new(RefCell::new(false));

        Panel::new(self)
            .with_caption("Supporting Kaspa NG")
            .with_back_enabled(core.has_stack(), |_|{
                *back.borrow_mut() = true;
            })
            .with_close_enabled(false, |_|{
            })
            // .with_header(|_ctx,ui| {
                
            // })
            .with_body(|_ctx,ui| {
                use egui_phosphor::light::CLIPBOARD_TEXT;
                
                ui.add_space(8.);                                    
                ui.label("This project relies on the support of the community.");
                ui.label(" ");
                ui.label("If you are able to contribute by donating, we would greatly appreciate your support.");
                ui.label(" ");
                ui.label("You can send donations to the following address:");
                ui.label(" ");
                
                ui.add(
                    Image::new(ImageSource::Bytes { uri : Cow::Owned(uri), bytes: qr })
                    .fit_to_original_size(1.0)
                    .texture_options(TextureOptions::NEAREST)
                );
                
                ui.label(" ");

                if ui
                    .add(Label::new(format!("{} {CLIPBOARD_TEXT}", format_address_string(Self::ADDRESS, Some(12)))).sense(Sense::click()))
                    .on_hover_ui_at_pointer(|ui|{
                        ui.vertical(|ui|{
                            ui.label("Click to copy the donation address to clipboard".to_string());
                        });
                    })
                    .clicked() {
                        ui.output_mut(|o| o.copied_text = Self::ADDRESS.to_owned());
                        runtime().notify(UserNotification::info(format!("{CLIPBOARD_TEXT} {}", i18n("Copied to clipboard"))).short())
                    }

                ui.label(" ");

            })
            .with_footer(|_this,ui| {
                if ui.large_button("Close").clicked() {
                    *back.borrow_mut() = true;
                }
            })
            .render(ui);

            if *back.borrow_mut() {
                core.back();
            }
    
    }

}
