use crate::imports::*;

pub struct Logs {
    #[allow(dead_code)]
    runtime: Runtime,
}

impl Logs {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            runtime,
        }
    }
}

impl ModuleT for Logs {

    // fn style(&self) -> ModuleStyle {
    //     ModuleStyle::Default
    // }

    fn render(
        &mut self,
        _core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        _ui: &mut egui::Ui,
    ) {

        #[cfg(not(target_arch = "wasm32"))]
        egui::ScrollArea::vertical()
            .id_source("node_logs")
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(_ui, |ui| {

                for log in self.runtime.kaspa_service().logs().iter() {
                    ui.label(RichText::from(log));
                }
            });


    }
}
