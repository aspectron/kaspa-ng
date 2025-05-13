use crate::imports::*;
use egui_phosphor::light::CLIPBOARD_TEXT;
use std::{borrow::Cow, collections::hash_map::Entry};

pub struct Donations {
    qr_kaspa_ng_fund : HashMap<String, (String,load::Bytes)>,
}

impl Donations {

    pub const ADDRESS_KASPA_NG_FUND: &'static str = "kaspa:qq2efzv0j7vt9gz9gfq44e6ggemjvvcuewhzqpm4ekf4fs5smruvs3c8ur9rp";

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
            //ui.output_mut(|o| Self::ADDRESS_KASPA_NG_FUND.clone_into(&mut o.copied_text));
            ui.ctx().copy_text(Self::ADDRESS_KASPA_NG_FUND.to_string());
            runtime().notify_clipboard(i18n("Copied to clipboard"));
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

                ui.label(i18n("The Kaspa NG software represents an ongoing effort focused on building a state-of-the-art software platform dedicated to the Kaspa BlockDAG cryptocurrency network. Ideological at its core, this software prioritizes security, privacy, performance, and decentralization."));
                ui.label(" ");
                ui.label(i18n("Because of its focus on security and performance, this software is entirely developed in Rust, demanding significantly more time and effort compared to other traditional modern web-driven software."));
                ui.label(" ");
                ui.label(i18n("We greatly appreciate your help in backing our efforts."));
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
