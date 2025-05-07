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

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        use egui_phosphor::light::CLIPBOARD_TEXT;

        let available_width = ui.available_width();

        #[cfg(not(target_arch = "wasm32"))]
        egui::ScrollArea::vertical()
            .id_salt("node_logs")
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {

                for log in self.runtime.kaspa_service().logs().iter() {
                    ui.label(RichText::from(log));
                }
            });

        let copy_to_clipboard = Button::new(RichText::new(format!(" {CLIPBOARD_TEXT} ")).size(20.));

        let button_rect = Rect::from_min_size(
            pos2(available_width - 48.0, core.device().top_offset() + 32.0),
            vec2(38.0, 20.0),
        );

        if ui.put(button_rect, copy_to_clipboard)
            .on_hover_text_at_pointer(i18n("Copy logs to clipboard"))
            .clicked() {
                let logs = self.runtime.kaspa_service().logs().iter().map(|log| log.to_string()).collect::<Vec<String>>().join("\n");
                //ui.output_mut(|o| o.copied_text = logs);
                ui.ctx().copy_text(logs);
                runtime().notify_clipboard(i18n("Copied to clipboard"));
            }
    }
}
