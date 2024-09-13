use crate::imports::*;
use super::*;

pub struct Destination<'context> {
    context : &'context mut ManagerContext,
}

impl<'context> Destination<'context> {
    pub fn new(context : &'context mut ManagerContext) -> Self {
        Self { context }
    }

    pub fn render(&mut self, _core: &mut Core, ui : &mut Ui, rc : &RenderContext) {
        let RenderContext { network_type, .. } = rc;

        TextEditor::new(
            &mut self.context.destination_address_string,
            // None,
            &mut self.context.focus,
            Focus::Address,
            |ui, text| {
                ui.add_space(8.);
                ui.label(RichText::new(i18n("Enter destination address")).size(12.).raised());
                ui.add_sized(Overview::editor_size(ui), TextEdit::singleline(text)
                    .vertical_align(Align::Center))
            },
        )
        .change(|address| {
            match Address::try_from(address) {
                Ok(address) => {
                    let address_network_type = NetworkType::try_from(address.prefix).expect("prefix to network type");
                    if address_network_type != *network_type {
                        self.context.address_status = AddressStatus::NetworkMismatch(address_network_type);
                    } else {
                        self.context.address_status = AddressStatus::Valid;
                    }
                }
                Err(err) => {
                    self.context.address_status = AddressStatus::Invalid(err.to_string());
                }
            }
        })
        .submit(|_, focus|{
            // *focus = Some(Focus::Amount);
            focus.next(Focus::Amount);
        })
        .build(ui);
        
        match &self.context.address_status {
            AddressStatus::Valid => {},
            AddressStatus::None => {},
            AddressStatus::NetworkMismatch(address_network_type) => {
                ui.label(i18n_args("This address if for the different network ({address_network_type})", &[("address_network_type", address_network_type.to_string())]));
            },
            AddressStatus::Invalid(err) => {
                ui.label(i18n_args("Please enter a valid address: {err}", &[("err", err)]));
            }
        }


    }
}