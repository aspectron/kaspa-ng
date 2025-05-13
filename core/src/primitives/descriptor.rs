use crate::imports::*;
use crate::primitives::transaction::paint_header_icon;
use convert_case::{Case, Casing};

// fn grid(ui: &mut Ui, id: &AccountId, add_contents: impl FnOnce(&mut Ui)) {
//     CollapsingHeader::new(id.to_string())
//         .default_open(true)
//         .show(ui, |ui| {
//             Grid::new("bip32_descriptor")
//                 .num_columns(3)
//                 .spacing([20.0, 4.0])
//                 .show(ui, |ui| {
//                     add_contents(ui);
//                 });
//         });
// }

pub trait RenderAccountDescriptor {
    fn render(&self, ui: &mut Ui, network: Network);
}

impl RenderAccountDescriptor for AccountDescriptor {
    fn render(&self, ui: &mut Ui, network: Network) {
        let collapsing_header = CollapsingHeader::new(self.account_id.to_string())
            .id_salt("bip32_descriptor")
            .icon(paint_header_icon)
            .default_open(true);

        collapsing_header.show(ui, |ui| {
            let default_color = theme_color().default_color;
            let color = theme_color().strong_color;

            let explorer = match network {
                Network::Mainnet => MAINNET_EXPLORER,
                Network::Testnet10 => TESTNET10_EXPLORER,
            };
            let pixels_per_point = ui.ctx().pixels_per_point();
            let one_char_width = ui
                .painter()
                .layout_no_wrap('x'.to_string(), Default::default(), color)
                .size()
                .x;
            let heading_width = one_char_width * 18.0;
            let width = ui.available_width() / pixels_per_point;
            let content = LayoutJobBuilderSettings::new(width, 8.0, None);
            ljb(&content)
                .heading(ui, heading_width, i18n("Account Name"), default_color)
                .text(
                    self.account_name.as_ref().unwrap_or(&"...".to_string()),
                    color,
                )
                .label(ui);
            ljb(&content)
                .heading(ui, heading_width, i18n("Type"), default_color)
                .text(
                    &self.account_kind().as_ref().to_case(Case::UpperCamel),
                    color,
                )
                .label(ui);

            let job =
                ljb(&content).heading(ui, heading_width, i18n("Receive Address"), default_color);
            match self.receive_address.as_ref() {
                Some(address) => {
                    job.address(
                        ui,
                        &address.to_string(),
                        &format!("{explorer}/addresses/{address}"),
                        color,
                        Some(6),
                    );
                }
                None => {
                    job.text("N/A", color).label(ui);
                }
            }

            let job =
                ljb(&content).heading(ui, heading_width, i18n("Change Address"), default_color);
            match self.change_address.as_ref() {
                Some(address) => {
                    job.address(
                        ui,
                        &address.to_string(),
                        &format!("{explorer}/addresses/{address}"),
                        color,
                        Some(6),
                    );
                }
                None => {
                    job.text("N/A", color).label(ui);
                }
            }

            for (prop, value) in self.properties.iter() {
                let text = value.to_string();
                let job = ljb(&content).heading(
                    ui,
                    heading_width,
                    i18n(prop.to_string().as_str()),
                    default_color,
                );

                if text.len() > 20 {
                    job.text(&format_partial_string(&text, Some(10)), color)
                        .with_clipboard_icon(ui, &text);
                } else {
                    job.text(&text, color).label(ui);
                }
            }

            //grid(ui, &self.account_id, |ui| {

            // ui.label(i18n("Account Name"));
            // ui.colored_label(
            //     color,
            //     self.account_name.as_ref().unwrap_or(&"...".to_string()),
            // );
            // ui.label("");
            // ui.end_row();
            // ui.label(i18n("Type"));
            // ui.colored_label(
            //     color,
            //     self.account_kind().as_ref().to_case(Case::UpperCamel),
            // );
            // ui.label("");
            // ui.end_row();
            //ui.label(i18n("Receive Address"));

            // match self.receive_address.as_ref(){
            //     Some(address)=>{
            //         ui.hyperlink_to_tab(
            //             RichText::new(format_address(address, None))
            //                 //.font(self.font_id.unwrap_or_default())
            //                 .color(color),
            //                 &format!("{explorer}/addresses/{address}"),
            //         );
            //         ui.label("xxxx");
            //         //LayoutJobBuilder::clipboard_icon(&mut ui[1], address.to_string());
            //     }
            //     None=>{
            //         ui.colored_label(
            //             color,
            //             "N/A".to_string()
            //         );
            //     }
            // }

            // ui.end_row();
            // ui.label(i18n("Change Address"));
            // match self.change_address.as_ref(){
            //     Some(address)=>{
            //         ui.hyperlink_to_tab(
            //             RichText::new(format_address(address, None))
            //                 //.font(self.font_id.unwrap_or_default())
            //                 .color(color),
            //                 &format!("{explorer}/addresses/{address}"),
            //         );
            //         LayoutJobBuilder::clipboard_icon(ui, address.to_string());
            //     }
            //     None=>{
            //         ui.colored_label(
            //             color,
            //             "N/A".to_string()
            //         );
            //     }
            // }

            // ui.end_row();

            // for (prop, value) in self.properties.iter() {
            //     ui.label(i18n(prop.to_string().as_str()));
            //     ui.colored_label(color, value.to_string());
            //     ui.label("");
            //     ui.end_row();
            // }
        });
    }
}
