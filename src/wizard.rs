use crate::imports::*;
pub use futures::{future::FutureExt, select, Future};

pub enum Container {
    Window(Box<dyn Fn(&egui::Context) -> egui::Window<'_>>),
    TopBottomPanel(Box<dyn Fn(&egui::Context) -> egui::TopBottomPanel>),
    CentralPanel(Box<dyn Fn(&egui::Context) -> egui::CentralPanel>),
    SidePanel(Box<dyn Fn(&egui::Context) -> egui::SidePanel>),
}

pub enum Disposition {
    Current,
    Previous,
    Next,
    Cancel,
}

pub trait WizardT {
    fn render_with_context(&mut self, ctx: &egui::Context) -> bool;
    fn render_with_ui(&mut self, ui: &mut egui::Ui) -> bool;
}

type FnStage<Ctx> = dyn Fn(&mut Ui, &mut Ctx) -> Disposition + 'static;
type FnFinish<Ctx> = dyn Fn(&mut Ctx) + 'static;

#[derive(Default)]
pub struct Wizard<Ctx> {
    ctx: Rc<RefCell<Ctx>>,
    stages: Vec<Rc<FnStage<Ctx>>>,
    finish: Option<Box<FnFinish<Ctx>>>,
    index: usize,
    container: Option<Container>,
}

impl<Ctx> Wizard<Ctx> {
    pub fn new() -> Self
    where
        Ctx: Default,
    {
        Self {
            ctx: Rc::new(RefCell::new(Ctx::default())),
            stages: vec![],
            finish: None,
            index: 0,
            container: None,
        }
    }

    pub fn with_context(ctx: Ctx) -> Self {
        Self {
            ctx: Rc::new(RefCell::new(ctx)),
            stages: vec![],
            finish: None,
            index: 0,
            container: None,
        }
    }

    pub fn with_window(
        mut self,
        ctor: impl Fn(&egui::Context) -> egui::Window<'_> + 'static,
    ) -> Self {
        self.container = Some(Container::Window(Box::new(ctor)));
        self
    }

    pub fn with_top_bottom_panel(
        mut self,
        ctor: impl Fn(&egui::Context) -> egui::TopBottomPanel + 'static,
    ) -> Self {
        self.container = Some(Container::TopBottomPanel(Box::new(ctor)));
        self
    }

    // pub fn stage<FnStageT>(
    pub fn stage(
    // pub fn stage(
        mut self,
        stage: impl Fn(&mut Ui, &mut Ctx) -> Disposition + 'static,
        // stage: &dyn Fn(&mut Ui, &mut Ctx) -> Disposition,
    ) -> Self
    // where
    //     FnStageT: Fn(&mut Ui, &mut Ctx) -> Disposition + 'static,
    {
        self.stages.push(Rc::new(stage));
        self
    }

    pub fn finish(mut self, finish: impl Fn(&mut Ctx) + 'static) // -> &mut Self
    where
        Ctx: 'static,
    {
        self.finish = Some(Box::new(finish));

        set_active_wizard(Some(Box::new(self)))

        // self
    }

    pub fn render_with_context_impl(&mut self, ctx: &egui::Context) -> bool {
        if self.index == self.stages.len() {
            let finish = self.finish.as_ref().expect("Missing `finish` phase");
            finish(&mut self.ctx.borrow_mut());

            set_active_wizard(None);

            true
        } else {
            let container = self
                .container
                .as_ref()
                .expect("Missing `container` (window or panel)");

            match container {
                Container::Window(window) => {
                    (window)(ctx).show(ctx, |ui| {
                        self.render_stage(ui);
                    });
                }
                Container::TopBottomPanel(panel) => {
                    (panel)(ctx).show(ctx, |ui| {
                        self.render_stage(ui);
                    });
                }
                Container::CentralPanel(panel) => {
                    (panel)(ctx).show(ctx, |ui| {
                        self.render_stage(ui);
                    });
                }
                Container::SidePanel(panel) => {
                    (panel)(ctx).show(ctx, |ui| {
                        self.render_stage(ui);
                    });
                }
            }

            false
        }
    }

    pub fn render_with_ui_impl(&mut self, ui: &mut egui::Ui) -> bool {
        if self.index == self.stages.len() {
            let finish = self.finish.as_ref().expect("Missing `finish` phase");
            finish(&mut self.ctx.borrow_mut());
            true
        } else {
            self.render_stage(ui);
            false
        }
    }

    fn render_stage(&mut self, ui: &mut Ui) {
        let stage = self.stages[self.index].clone();
        match stage(ui, &mut self.ctx.borrow_mut()) {
            Disposition::Previous => {
                if self.index > 0 {
                    self.index -= 1;
                } else {
                    panic!("Wizard `Disposition::Previous` invoked on the first stage")
                }
            }
            Disposition::Next => {
                self.index += 1;
            }
            Disposition::Current => {}
            Disposition::Cancel => {
                set_active_wizard(None);
            }
        }
    }
}

impl<Ctx> WizardT for Wizard<Ctx> {
    fn render_with_context(&mut self, ctx: &egui::Context) -> bool {
        self.render_with_context_impl(ctx)
    }
    fn render_with_ui(&mut self, ctx: &mut egui::Ui) -> bool {
        self.render_with_ui_impl(ctx)
    }
}

// - -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
// - =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// - -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
// - =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=

static mut WIZARD: Option<Box<dyn WizardT>> = None;

pub fn wizard() -> Option<&'static mut Box<dyn WizardT>> {
    unsafe {
        if WIZARD.is_none() {
            None
        } else {
            WIZARD.as_mut()
        }
    }
}

fn set_active_wizard(wizard: Option<Box<dyn WizardT>>) {
    unsafe {
        WIZARD = wizard;
    }
}

