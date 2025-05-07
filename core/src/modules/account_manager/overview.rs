use crate::imports::*;
use super::*;

pub struct Overview<'manager> {
    context : &'manager mut ManagerContext,
}

impl<'manager> Overview<'manager> {
    pub fn new(context : &'manager mut ManagerContext) -> Self {
        Self { context }
    }

    pub fn editor_size(ui : &Ui) -> Vec2 {
        Vec2::new(ui.available_width() * 0.75, 32.)
    }

    pub fn render(&mut self, core: &mut Core, ui : &mut Ui, rc : &RenderContext) {
        use egui_phosphor::light::{ARROW_CIRCLE_UP,ARROWS_DOWN_UP,QR_CODE};

        core.apply_mobile_style(ui);

        ui.add_space(8.);

        egui::ScrollArea::vertical()
            .id_salt("overview_metrics")
            .auto_shrink([false; 2])
            .show(ui, |ui| {

                ui.vertical_centered(|ui| {

                    AddressPane::new(self.context).render(core, ui, rc);
                    BalancePane::new(self.context).render(core, ui, rc);

                    if !core.state().is_synced() || !core.state().is_connected() {
                        NetworkState::new(self.context).render(core, ui, rc);
                        return;
                    }

                    match self.context.action.clone() {
                        Action::Sending | Action::Estimating | Action::Processing => {
                            Processor::new(self.context).render(core, ui, rc);
                            // self.render_send_ui(core, ui, rc);
                        }
                        Action::Error(error) => {
                            ui.vertical_centered(|ui|{

                                ui.add_space(16.);
                                ui.label(RichText::new(i18n("An error has occurred while submitting transaction:")));
                                ui.add_space(16.);
                                ui.separator();
                                ui.add_space(8.);
                                ui.label(RichText::new(error.to_string()).color(theme_color().error_color));
                                ui.add_space(8.);
                                ui.separator();
                                ui.add_space(16.);
                                // TODO - copy to clipboard?
                                if ui.medium_button(i18n("Continue")).clicked() {
                                    self.context.action = Action::None;
                                }
                            });
                        }
                        Action::None => {

                            Qr::render(ui, rc);

                            ui.vertical_centered(|ui|{
                            
                                ui.add_space(8.);
                                ui.horizontal(|ui| {

                                    let mut layout = CenterLayoutBuilder::new();
                                    
                                    layout = layout.add(Button::new(i18n_args("{arrowCircleUpIcon} Send", &[("arrowCircleUpIcon", ARROW_CIRCLE_UP)])).min_size(theme_style().medium_button_size()), |(this, _):&mut (&mut Overview<'_>, &mut Core)| {
                                        this.context.action = Action::Estimating;
                                        this.context.transaction_kind = Some(TransactionKind::Send);
                                        this.context.focus.next(Focus::Address);
                                    });

                                    if core.account_collection().as_ref().map(|collection|collection.len()).unwrap_or(0) > 1 {
                                        layout = layout.add(Button::new(i18n_args("{arrowsDownUpIcon} Transfer", &[("arrowsDownUpIcon", ARROWS_DOWN_UP)])).min_size(theme_style().medium_button_size()), |(this,_)| {
                                            this.context.action = Action::Estimating;
                                            this.context.transaction_kind = Some(TransactionKind::Transfer);
                                            this.context.focus.next(Focus::Amount);
                                        });
                                    }
                                    layout = layout.add(Button::new(i18n_args("{qrCodeIcon} Request", &[("qrCodeIcon", QR_CODE)])).min_size(theme_style().medium_button_size()), |(_,core)| {
                                        core.get_mut::<modules::Request>().select(&rc.account);
                                        core.select::<modules::Request>();
                        
                                    });

                                    layout.build(ui,&mut (self,core));
                                });
                            });
                        }
                    }
                });
            });
    }

}