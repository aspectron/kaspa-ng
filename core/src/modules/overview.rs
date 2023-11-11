use egui_extras::{StripBuilder, Size};

use crate::imports::*;

pub struct Overview {
    #[allow(dead_code)]
    interop: Interop,
}

impl Overview {
    pub fn new(interop: Interop) -> Self {
        Self { interop }
    }
}

impl ModuleT for Overview {
    fn render(
        &mut self,
        core: &mut Core,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        StripBuilder::new(ui)
            .size(Size::remainder())
            .size(Size::exact(10.0))
            .size(Size::remainder())
            // .size(Size::relative(0.5).at_least(60.0))
            // .size(Size::exact(10.5))
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    ui.label("Kaspa NG");
                });
                strip.cell(|ui| {
                    ui.separator();
                });
                strip.cell(|ui| {
                    ui.label("Wallet Stuff");
                    // let cell = self.modules.get(&TypeId::of::<T>()).unwrap();

                    let module = core.modules().get(&TypeId::of::<modules::AccountManager>()).unwrap().clone();
                    module.render_default(core,ctx,frame,ui);
                });
            });


        // ui.horizontal(|ui| {
        //     ui.label("Kaspa NG");
        //     ui.separator();
        //     ui.label("Overview");
        // });

    }
}
