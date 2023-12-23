use crate::imports::*;

#[derive(Default)]
pub struct NetworkInterfaceEditor {
    kind: NetworkInterfaceKind,
    custom: String,
}

impl NetworkInterfaceEditor {
    pub fn new(network_interface: &NetworkInterfaceConfig) -> Self {
        Self {
            kind: network_interface.kind,
            custom: network_interface.custom.to_string(),
        }
    }
}

impl From<&NetworkInterfaceConfig> for NetworkInterfaceEditor {
    fn from(network_interface_config: &NetworkInterfaceConfig) -> Self {
        NetworkInterfaceEditor {
            kind: network_interface_config.kind,
            custom: network_interface_config.custom.to_string(),
        }
    }
}

impl TryFrom<&NetworkInterfaceEditor> for NetworkInterfaceConfig {
    type Error = Error;
    fn try_from(network_interface_editor: &NetworkInterfaceEditor) -> Result<Self> {
        Ok(NetworkInterfaceConfig {
            kind: network_interface_editor.kind,
            custom: network_interface_editor.custom.parse()?,
        })
    }
}

impl AsRef<NetworkInterfaceEditor> for NetworkInterfaceEditor {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl NetworkInterfaceEditor {
    pub fn is_valid(&self) -> bool {
        match self.kind {
            NetworkInterfaceKind::Custom => NetworkInterfaceConfig::try_from(self).is_ok(),
            _ => true,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(i18n("Network Interface"));
                ui.radio_value(
                    &mut self.kind,
                    NetworkInterfaceKind::Local,
                    "Local (127.0.0.1)",
                );
                ui.radio_value(&mut self.kind, NetworkInterfaceKind::Any, "Any (0.0.0.0)");
                ui.radio_value(&mut self.kind, NetworkInterfaceKind::Custom, "Custom");
            });

            if self.kind == NetworkInterfaceKind::Custom {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(i18n("Custom Network Interface:"));
                        ui.text_edit_singleline(&mut self.custom);
                    });
                    if self.custom.is_empty() {
                        ui.label(
                            RichText::new(i18n("Please enter custom interface address: IP[:PORT]"))
                                .color(theme_color().warning_color),
                        );
                    } else if let Err(err) = ContextualNetAddress::from_str(self.custom.as_str()) {
                        ui.label(
                            RichText::new(format!("Error: {}", err))
                                .color(theme_color().error_color),
                        );
                    }
                });
            }
        });
    }
}
