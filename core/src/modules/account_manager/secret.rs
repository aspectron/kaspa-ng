use crate::imports::*;
use super::*;

pub struct AccountSecret<'render> {
    core : &'render mut Core,
    manager : &'render AccountManager,
}

impl<'render> AccountSecret<'render> {

    pub fn new(core : &'render mut Core, manager : &'render AccountManager) -> Self {
        Self { core, manager }
    }

    pub fn render(&mut self, ui : &mut Ui, rc : &RenderContext<'_>) {

        
    }


}
