use crate::imports::*;
use convert_case::{Case, Casing};
// #[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
// pub struct Bip32 {
//     pub account_id: AccountId,
//     pub account_name: Option<String>,
//     pub prv_key_data_id: PrvKeyDataId,
//     pub account_index: u64,
//     pub xpub_keys: Arc<Vec<String>>,
//     pub ecdsa: bool,
//     pub receive_address: Option<Address>,
//     pub change_address: Option<Address>,
//     pub meta: AddressDerivationMeta,
// }

// #[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
// pub struct Keypair {
//     pub account_id: AccountId,
//     pub account_name: Option<String>,
//     pub prv_key_data_id: PrvKeyDataId,
//     pub public_key: String,
//     pub ecdsa: bool,
//     pub receive_address: Option<Address>,
//     pub change_address: Option<Address>,
// }

// #[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
// pub struct Legacy {
//     pub account_id: AccountId,
//     pub account_name: Option<String>,
//     pub prv_key_data_id: PrvKeyDataId,
//     // pub xpub_keys: Arc<Vec<String>>,
//     pub receive_address: Option<Address>,
//     pub change_address: Option<Address>,
//     pub meta: AddressDerivationMeta,
// }

// #[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
// pub struct MultiSig {
//     pub account_id: AccountId,
//     pub account_name: Option<String>,
//     // TODO add multisig data
// }

// #[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
// pub struct Resident {
//     pub account_id: AccountId,
//     pub account_name: Option<String>,
//     pub public_key: String,
// }

// use kaspa_wallet_core::account::descriptor::*;

fn grid(ui: &mut Ui, id: &AccountId, add_contents: impl FnOnce(&mut Ui)) {
    CollapsingHeader::new(id.to_string())
        .default_open(true)
        .show(ui, |ui| {
            Grid::new("bip32_descriptor")
                .num_columns(2)
                .spacing([20.0, 4.0])
                // .min_col_width(120.0)
                // .striped(true)
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
            // TODO bip39 info
            // ui.label(i18n("Derivation Index"));
            // ui.colored_label(color, format!("BIP-44 / {}", self.account_index));
            // ui.end_row();
            // ui.label(i18n("Signature Type"));
            // ui.colored_label(color, if self.ecdsa { "ECDSA" } else { "Schnorr" });
            // ui.end_row();
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
