use crate::imports::*;
use crate::egui::easy_mark;
pub struct Changelog {
    #[allow(dead_code)]
    runtime: Runtime,
    changelog : &'static str,
}

impl Changelog {
    pub fn new(runtime: Runtime) -> Self {

        Self { 
            runtime,
            changelog : include_str!("../../../CHANGELOG.md")
        }
    }
}

impl ModuleT for Changelog {

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Mobile
    }

    fn render(
        &mut self,
        _core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        egui::ScrollArea::vertical()
            .id_source("changelog")

            .auto_shrink([false; 2])
            .show(ui, |ui| {
                easy_mark(ui, self.changelog.as_str());
            });
    }
}
