use crate::imports::*;
use super::*;

pub struct NetworkState<'context> {
    #[allow(dead_code)]
    pub context: &'context ManagerContext,
}

impl<'context> NetworkState<'context> {
    pub fn new(context: &'context ManagerContext) -> Self {
        Self { context }
    }

    pub fn render(&mut self, core: &mut Core, ui: &mut Ui, rc: &RenderContext) {

        use egui_phosphor::light::{CLOUD_SLASH,CLOUD_ARROW_DOWN};

        const ICON_SPACING: f32 = 24.0;
        ui.vertical_centered(|ui|{
            // ui.add_space(16.);
            if !core.state().is_connected() {
                ui.add_space(ICON_SPACING);
                ui.label(
                    RichText::new(CLOUD_SLASH)
                        .size(theme_style().icon_size_large)
                        .color(theme_color().icon_color_default)
                );
                ui.add_space(ICON_SPACING);
                
                ui.label(i18n("You are currently not connected to the Kaspa node."));
            } else if !core.state().is_synced() {
                
                ui.add_space(ICON_SPACING);
                ui.label(
                    RichText::new(CLOUD_ARROW_DOWN)
                        .size(theme_style().icon_size_large)
                        .color(theme_color().icon_color_default)
                );
                ui.add_space(ICON_SPACING);

                ui.label(i18n("The node is currently syncing with the Kaspa p2p network."));
                ui.add_space(16.);
                ui.label(i18n("Please wait for the node to sync or connect to a remote node."));
            }
            ui.add_space(16.);
            ui.label(i18n("You can configure remote connection in Settings"));
            ui.add_space(16.);
            if ui.large_button(i18n("Go to Settings")).clicked() {
                core.select::<modules::Settings>();
            }
            ui.label("");
            if ui.large_button(i18n("Payment Request")).clicked() {
                core.get_mut::<modules::Request>().select(&rc.account);
                core.select::<modules::Request>();
            }
        });

    }
}