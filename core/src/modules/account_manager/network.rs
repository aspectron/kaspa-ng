use crate::imports::*;
use super::*;

pub struct NetworkState<'context> {
    pub context: &'context ManagerContext,
}

impl<'context> NetworkState<'context> {
    pub fn new(context: &'context ManagerContext) -> Self {
        Self { context }
    }

    pub fn render(&mut self, core: &mut Core, ui: &mut Ui, _rc: &RenderContext<'_>) {

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
                
                ui.label("You are currently not connected to the Kaspa node.");
            } else if !core.state().is_synced() {
                
                ui.add_space(ICON_SPACING);
                ui.label(
                    RichText::new(CLOUD_ARROW_DOWN)
                        .size(theme_style().icon_size_large)
                        .color(theme_color().icon_color_default)
                );
                ui.add_space(ICON_SPACING);

                ui.label("The node is currently syncing with the Kaspa p2p network.");
                ui.add_space(16.);
                ui.label("Please wait for the node to sync or connect to a remote node.");
            }
            ui.add_space(16.);
            ui.label("You can configure remote connection in Settings");
            ui.add_space(16.);
            if ui.large_button("Go to Settings").clicked() {
                core.select::<modules::Settings>();
            }
        });

    }
}