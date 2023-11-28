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
        ui: &mut egui::Ui,
    ) {
        use egui_phosphor::light::CLIPBOARD_TEXT;

        #[cfg(not(target_arch = "wasm32"))]
        egui::ScrollArea::vertical()
            .id_source("node_logs")
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {

                for log in self.runtime.kaspa_service().logs().iter() {
                    ui.label(RichText::from(log));
                }
            });

        let copy_to_clipboard = Button::new(format!(" {CLIPBOARD_TEXT} "));

        let screen_rect = ui.ctx().screen_rect();
        let button_rect = Rect::from_min_size(
            pos2(screen_rect.max.x - 48.0, screen_rect.min.y + 32.0),
            vec2(38.0, 20.0),
        );

        if ui.put(button_rect, copy_to_clipboard)
            .on_hover_text_at_pointer(i18n("Copy logs to clipboard"))
            .clicked() {
                let logs = self.runtime.kaspa_service().logs().iter().map(|log| log.to_string()).collect::<Vec<String>>().join("\n");
                ui.output_mut(|o| o.copied_text = logs);
                runtime().notify(UserNotification::info(i18n("Copied to clipboard")).short())
            }
    }
}
