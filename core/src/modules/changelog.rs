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
            changelog : include_str!("../../CHANGELOG.md")
        }
    }
}

impl ModuleT for Changelog {

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Mobile
    }

    fn modal(&self) -> bool {
        true
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        let max_height = ui.available_height() - 64.;

        ui.vertical(|ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(max_height)
                .show(ui, |ui| {
                    easy_mark(ui, self.changelog);
                });

            ui.vertical_centered(|ui|{
                ui.separator();
                ui.add_space(8.);
                if ui.large_button(i18n("Close")).clicked() {
                    core.select::<modules::Overview>();
                }
            });
        });

    }
}
