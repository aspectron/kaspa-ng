use crate::imports::*;

pub struct Request {
    #[allow(dead_code)]
    runtime: Runtime,
}

impl Request {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }
}

impl ModuleT for Request {
    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        Panel::new(self)
        .with_caption("Payment Request")
        .with_back_enabled(core.has_stack(), |_|{
            core.back();
        })
        // .with_close_enabled(core.has_stack(), |_| {
        //     core.back();
        // })
        .with_header(|_ctx, _ui| {
            // ui.label(text);
        })
        .with_body(|_this, ui| {

            ui.label("");
            ui.label(i18n("Payment request panel"));
            ui.label("");
        })
        .render(ui);
    }
}
