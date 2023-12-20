use crate::imports::*;
use convert_case::{Case, Casing};

fn grid(ui: &mut Ui, id: &AccountId, add_contents: impl FnOnce(&mut Ui)) {
    CollapsingHeader::new(id.to_string())
        .default_open(true)
        .show(ui, |ui| {
            Grid::new("bip32_descriptor")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    add_contents(ui);
                });
        });
}

pub trait RenderAccountDescriptor {
    fn render(&self, ui: &mut Ui);
}

impl RenderAccountDescriptor for AccountDescriptor {
    fn render(&self, ui: &mut Ui) {
        grid(ui, &self.account_id, |ui| {
            let color = Color32::WHITE;

            ui.label(i18n("Account Name"));
            ui.colored_label(
                color,
                self.account_name.as_ref().unwrap_or(&"...".to_string()),
            );
            ui.end_row();
            ui.label(i18n("Type"));
            ui.colored_label(
                color,
                self.account_kind().as_ref().to_case(Case::UpperCamel),
            );
            ui.end_row();
            ui.label(i18n("Receive Address"));
            ui.colored_label(
                color,
                self.receive_address
                    .as_ref()
                    .map(String::from)
                    .unwrap_or("N/A".to_string()),
            );
            ui.end_row();
            ui.label(i18n("Change Address"));
            ui.colored_label(
                color,
                self.change_address
                    .as_ref()
                    .map(String::from)
                    .unwrap_or("N/A".to_string()),
            );
            ui.end_row();

            for (prop, value) in self.properties.iter() {
                ui.label(i18n(prop.to_string().as_str()));
                ui.colored_label(color, value.to_string());
                ui.end_row();
            }
        });
    }
}
