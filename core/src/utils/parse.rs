use crate::imports::*;

pub fn try_user_string_to_address(address: &str, network_type: &NetworkType) -> Result<Address> {
    let address = Address::try_from(address)?;
    let address_network_type = NetworkType::try_from(address.prefix)?;
    if &address_network_type != network_type {
        return Err(Error::custom(format!(
            "{} {} ({} {})",
            i18n("Invalid address network type:"),
            address_network_type,
            i18n("expected:"),
            network_type
        )));
    }
    Ok(address)
}
