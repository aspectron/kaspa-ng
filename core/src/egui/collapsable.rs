use crate::imports::*;
use egui::collapsing_header::CollapsingState;

pub trait CollapsingExtension {
    fn collapsable<HeaderRet, BodyRet>(
        &mut self,
        id: impl Into<String>,
        default_open: bool,
        heading: impl FnOnce(&mut Ui, &mut bool) -> HeaderRet,
        body: impl FnOnce(&mut Ui) -> BodyRet,
    );
}

impl CollapsingExtension for Ui {
    fn collapsable<HeaderRet, BodyRet>(
        &mut self,
        id: impl Into<String>,
        default_open: bool,
        heading: impl FnOnce(&mut Ui, &mut bool) -> HeaderRet,
        body: impl FnOnce(&mut Ui) -> BodyRet,
    ) {
        let id: String = id.into();
        let id = self.make_persistent_id(id);
        let previous_state = CollapsingState::load(self.ctx(), id)
            .map(|state| state.is_open())
            .unwrap_or_default();
        let mut state = previous_state;
        let header = CollapsingState::load_with_default_open(self.ctx(), id, default_open);
        header
            .show_header(self, |ui| heading(ui, &mut state))
            .body(body);

        if state != previous_state {
            if let Some(mut state) = CollapsingState::load(self.ctx(), id) {
                state.toggle(self);
                state.store(self.ctx());
            }
        }
    }
}
