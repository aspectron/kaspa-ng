use crate::imports::*;

pub struct Logs {
    #[allow(dead_code)]
    interop: Interop,
}

impl Logs {
    pub fn new(interop: Interop) -> Self {
        Self {
            interop,
        }
    }
}

impl ModuleT for Logs {

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Default
    }

    fn render(
        &mut self,
        _wallet: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        egui::ScrollArea::vertical()
            .id_source("node_logs")
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {

                for log in self.interop.kaspa_service().logs().iter() {
                    ui.label(RichText::from(log));
                }
            });


    }
}
