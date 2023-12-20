use crate::imports::*;
use crate::egui::easy_mark;
use egui_phosphor::light::X;
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
        core: &mut Core,
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

            let close = Label::new(RichText::new(format!(" {X} ")).size(20.)).sense(Sense::click());

            let screen_rect = ui.ctx().screen_rect();
            let close_rect = Rect::from_min_size(
                pos2(screen_rect.max.x - 48.0, screen_rect.min.y + 32.0),
                vec2(42.0, 42.0),
            );
    
            if ui.put(close_rect, close)
                .clicked() {
                    core.select::<modules::Overview>();
                }
    
    }
}
