use crate::imports::*;
use super::*;

pub struct AddressPane<'context> {
    #[allow(dead_code)]
    context : &'context ManagerContext,
}

impl<'context> AddressPane<'context> {
    pub fn new(context : &'context ManagerContext) -> Self {
        Self { context }
    }

    pub fn render(&mut self, _core: &mut Core, ui : &mut Ui, rc : &RenderContext) {
        use egui_phosphor::light::CLIPBOARD_TEXT;
        let address = format_address(rc.context.address(), Some(8));
        if ui.add(Label::new(format!("Address: {address} {CLIPBOARD_TEXT}")).sense(Sense::click()))
            // .on_hover_ui_at_pointer(|ui|{
            //     ui.vertical(|ui|{
            //         ui.add(Label::new(format!("{}", context.address().to_string())));
            //         ui.add_space(16.);
            //         ui.label("Click to copy address to clipboard".to_string());
            //     });
            // })
            .clicked() {
                //ui.output_mut(|o| o.copied_text = rc.context.address().to_string());
                ui.ctx().copy_text(rc.context.address().to_string());
                runtime().notify_clipboard(i18n("Copied to clipboard"));
            }
    }
}