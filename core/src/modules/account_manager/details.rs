use crate::imports::*;
use super::*;

pub struct Details {
}

impl Details {
    pub fn new() -> Self {
        Self { }
    }

    pub fn render(&mut self, _core: &mut Core, ui : &mut Ui, rc : &RenderContext) {
        let RenderContext { account, .. } = rc;

        egui::ScrollArea::vertical().auto_shrink([false,false]).show(ui, |ui| {

            let descriptor = account.descriptor();

            descriptor.render(ui, account.network());
            ui.add_space(8.);

            let mut address_kind : Option<NewAddressKind> = None;
            
            ui.horizontal(|ui|{
                if ui.medium_button(i18n("Generate New Receive Address")).clicked() {
                    address_kind = Some(NewAddressKind::Receive);
                }
                if ui.medium_button(i18n("Generate New Change Address")).clicked() {
                    address_kind = Some(NewAddressKind::Change);
                }
            });

            if let Some(address_kind) = address_kind {
                let account_id = account.id();
                spawn(async move {
                    runtime()
                        .wallet()
                        .accounts_create_new_address(account_id, address_kind)
                        .await
                        .map_err(|err|Error::custom(i18n_args("Failed to create new address: {err}",&[("err",err.to_string())])))?;

                    runtime().request_repaint();

                    Ok(())
                });
            }
        });       
    }
}