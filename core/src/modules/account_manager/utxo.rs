use crate::imports::*;
use super::*;

pub struct UtxoManager {
}

impl UtxoManager {
    pub fn new() -> Self {
        Self { }
    }

    pub fn render(&mut self, _core: &mut Core, ui : &mut Ui, rc : &RenderContext<'_>) {
        let RenderContext { account: _, .. } = rc;

        egui::ScrollArea::vertical().auto_shrink([false,false]).show(ui, |ui| {
            ui.label("UTXO Manager");
        });

    }
}