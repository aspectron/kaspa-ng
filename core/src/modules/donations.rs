use crate::imports::*;
use egui_phosphor::light::CLIPBOARD_TEXT;
use std::{borrow::Cow, collections::hash_map::Entry};

pub struct Donations {
    qr_kaspa_ng_fund : HashMap<String, (String,load::Bytes)>,
}

impl Donations {

    pub const ADDRESS_KASPA_NG_FUND: &'static str = "kaspatest:qqdr2mv4vkes6kvhgy8elsxhvzwde42629vnpcxe4f802346rnfkklrhz0x7x";

    pub fn new(_runtime: Runtime) -> Self {
        Self { 
            qr_kaspa_ng_fund : Default::default(),
        }
    }

    fn qr_kaspa_ng_fund(&mut self) -> (String,load::Bytes) {

        let (uri,qr) = match self.qr_kaspa_ng_fund.entry(theme_color().name.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let uri = format!("bytes://{}-{}.svg", Self::ADDRESS_KASPA_NG_FUND, theme_color().name);
                let qr = render_qrcode(Self::ADDRESS_KASPA_NG_FUND, 128, 128);
                entry.insert((uri, qr.as_bytes().to_vec().into()))
            },
        };

        (uri.clone(),qr.clone())
    }

    fn render_destination(&self, ui: &mut Ui, (uri, qr) : (String,load::Bytes) ) {
        ui.add(
            Image::new(ImageSource::Bytes { uri : Cow::Owned(uri), bytes: qr })
            .fit_to_original_size(1.0)
            .texture_options(TextureOptions::NEAREST)
        );
        
        ui.label(" ");

        let response = ui.add(Label::new(format!("{} {CLIPBOARD_TEXT}", format_address_string(Self::ADDRESS_KASPA_NG_FUND, Some(12)))).sense(Sense::click()))
        .on_hover_ui_at_pointer(|ui|{
            ui.vertical(|ui|{
                ui.label(i18n("Click to copy the donation address to clipboard"));
            });
        });

        if response.clicked() {
            ui.output_mut(|o| o.copied_text = Self::ADDRESS_KASPA_NG_FUND.to_owned());
            runtime().notify(UserNotification::info(format!("{CLIPBOARD_TEXT} {}", i18n("Copied to clipboard"))).short());
        }

        ui.label(" ");

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
        let back = Rc::new(RefCell::new(false));

        Panel::new(self)
            .with_caption("Supporting Kaspa NG")
            .with_back_enabled(core.has_stack(), |_|{
                *back.borrow_mut() = true;
            })
            .with_close_enabled(false, |_|{
            })
            .with_body(|this,ui| {
                
                ui.add_space(8.);                                    
                ui.label(i18n("This project relies on the support of the community."));
                ui.label(" ");
                ui.label(i18n("If you are able to contribute by donating, we would greatly appreciate your support."));
                ui.label(" ");
                ui.label(i18n("You can send donations to the following addresses:"));
                ui.label(" ");

                ui.separator();
                ui.label(" ");
                ui.label(i18n("Kaspa NG development fund"));
                ui.label(" ");
                let kaspa_ng_fund = this.qr_kaspa_ng_fund();
                this.render_destination(ui, kaspa_ng_fund);
                ui.label(" ");

            })
            .with_footer(|_this,ui| {
                if ui.large_button(i18n("Close")).clicked() {
                    *back.borrow_mut() = true;
                }
            })
            .render(ui);

            if *back.borrow_mut() {
                core.back();
            }
    
    }

}
