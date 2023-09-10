// use std::marker::PhantomData;

use crate::imports::*;
pub use futures::{future::FutureExt, select, Future};

pub enum Container {
    // None,
    Window(Box<dyn Fn(&egui::Context) -> egui::Window<'_>>),
    TopBottomPanel(Box<dyn Fn(&egui::Context) -> egui::TopBottomPanel>),
    // TopBottomPanel(egui::TopBottomPanel),
}

pub enum Disposition {
    Current,
    Previous,
    Next,
}

trait BindingT {
    fn render(&mut self, ctx: &egui::Context) -> bool;
    // fn render(&mut self, ui : &mut Ui) -> bool;
}

// pub trait ContainerT<'wizard> {
// pub trait ContainerT {
//     fn show_impl(self, ctx: &egui::Context, show_fn : Box<dyn FnMut(&mut Ui)>);
//     // fn show_impl(&self, ctx: &egui::Context, show_fn : impl FnMut(&mut Ui));
//     // fn show_impl(&self, ctx: &egui::Context, binding : &mut Box<dyn BindingT>); //, show_fn : impl FnMut(&mut Ui));
// }

// // impl<'wizard> ContainerT<'wizard> for egui::Window<'wizard> {
//     impl ContainerT for egui::Window<'_> {
//     // fn show_impl(&self, ctx: &egui::Context, binding : &mut Box<dyn BindingT>) { //, show_fn : impl FnMut(&mut Ui));
//     fn show_impl(self, ctx: &egui::Context, mut show_fn : Box<dyn FnMut(&mut Ui)>) {
//     // fn show_impl(&self, ctx: &egui::Context, mut show_fn : impl FnMut(&mut Ui)) {
//         self.show(ctx, move |ui| {
//             // binding.render(ui)
//             show_fn(ui);
//         });

//     }
// }

// - BINDING SHOULD BE FOR WIZARD, NOT FOR FUNCTION WRAPPER!
// - WIZARD SHOULD CONTAIN A LIST OF FUNCTIONS WITH CTX

type FnStage<Ctx> = dyn Fn(&mut Ui, &mut Ctx) -> Disposition + 'static;
type FnFinish<Ctx> = dyn Fn(&mut Ctx) + 'static;
// type FnContainer<Container> = dyn Fn(&mut egui::Context) -> Container + 'static;
// type FnContainerT<'wizard> = dyn Fn(&mut egui::Context) -> Box<dyn ContainerT<'wizard>>;
// type FnContainerT = dyn Fn(&egui::Context) -> Box<dyn ContainerT> + 'static;

// pub struct Wizard<Ctx, FnRender>
#[derive(Default)]
pub struct Wizard<Ctx>
// where    Ctx : 'static
{
    ctx: Rc<RefCell<Ctx>>,
    // binding: Vec<Rc<dyn BindingT>>,
    // binding: Vec<Rc<FnRender>>,
    // stages = Vec<Rc<dyn Fn(&mut Ui, &mut Ctx) -> Disposition + 'static>>,
    stages: Vec<Rc<FnStage<Ctx>>>,
    finish: Option<Box<FnFinish<Ctx>>>,
    index: usize,
    container: Option<Container>, //Rc<Box<FnContainerT>>,
}

impl<Ctx> Wizard<Ctx>
// where
// FnRender: Fn(&mut Ui, &mut Ctx) -> Disposition + 'static,
// Ctx : 'static
{
    // pub fn new(container : impl Fn(&mut egui::Context) -> Box<dyn ContainerT> + 'static) -> Self
    // pub fn new_window(ctor : impl Fn(&egui::Context) -> egui::Window<'_> + 'static) -> Self
    pub fn new() -> Self
    where
        Ctx: Default,
        // FnContainer : impl Fn(&mut egui::Context) -> Box<dyn ContainerT>
    {
        Self {
            ctx: Rc::new(RefCell::new(Ctx::default())),
            stages: vec![],
            finish: None,
            index: 0,
            // container : Container::Window(Box::new(ctor)) //Rc::new(Box::new(container)),
            container: None,
            // container : None,
        }
    }

    pub fn with_context(ctx: Ctx) -> Self {
        // }, container : impl Fn(&egui::Context) -> Box<dyn ContainerT> + 'static) -> Self {
        Self {
            ctx: Rc::new(RefCell::new(ctx)),
            stages: vec![],
            finish: None,
            index: 0,
            container: None, //Rc::new(Box::new(container)),
                             // container : None
        }
    }

    pub fn with_window(
        &mut self,
        ctor: impl Fn(&egui::Context) -> egui::Window<'_> + 'static,
    ) -> &mut Self {
        self.container = Some(Container::Window(Box::new(ctor)));
        self
    }

    pub fn with_top_bottom_panel(
        &mut self,
        ctor: impl Fn(&egui::Context) -> egui::TopBottomPanel + 'static,
    ) -> &mut Self {
        self.container = Some(Container::TopBottomPanel(Box::new(ctor)));
        self
    }

    // pub fn with_window(&mut self, window: egui::Window) -> &mut Self {
    //     self.container = Some(Container::Window(window));
    //     self
    // }

    pub fn stage<FnStageT>(
        &mut self,
        stage: impl Fn(&mut Ui, &mut Ctx) -> Disposition + 'static,
    ) -> &mut Self
    where
        FnStageT: Fn(&mut Ui, &mut Ctx) -> Disposition + 'static,
        //     Ctx: 'static,
    {
        // let binding = Binding {
        //     ctx : self.ctx.clone(),
        //     render,
        // };

        // let binding: Rc<dyn BindingT> = Rc::new(binding);
        // let stage: Rc<dyn Fn(&mut Ui, &mut Ctx) -> Disposition + 'static> = Rc::new(stage);

        // self.stages.push(stage);
        self.stages.push(Rc::new(stage));

        // self.binding.push(binding);

        self
    }

    pub fn finish(&mut self, finish: impl Fn(&mut Ctx) + 'static) -> &mut Self
    where
        // FnFinish: Fn(&mut Ctx) + 'static,
        Ctx: 'static,
    {
        // let finish: Box<FnFinish<Ctx>> = Box::new(finish);
        // let finish: Box<FnFinish<Ctx>> = Box::new(finish);
        self.finish = Some(Box::new(finish));
        // let binding = Binding {
        //     ctx : self.ctx.clone(),
        //     render,
        // };

        // let binding: Rc<dyn BindingT> = Rc::new(binding);

        // self.binding.push(binding);

        self
    }

    pub fn render_impl(&mut self, ctx: &egui::Context) -> bool {
        // let mut disposition = Disposition::Current;
        // let mut ui = egui::Ui::__test();

        if self.index == self.stages.len() {
            // - TODO - CALL FINAL FN
            // - TODO - CALL FINAL FN
            // - TODO - CALL FINAL FN
            // - TODO - CALL FINAL FN

            true
        } else {
            let container = self.container.as_ref().unwrap();

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
            }
            // let container = self.container.clone();
            // let container = (container)(ctx);
            // container.show_impl(ctx, Box::new(move |ui| {
            //     self.render_stage(ui);

            // }));
            // let container = self.container.as_ref().expect("Missing wizard container");
            // match container {
            //     Container::Window(window) => {
            //         window.show(ctx, |ui| {
            //             self.render_stage(ui);
            //         });
            //     }
            //     Container::TopBottomPanel(panel) => {
            //         panel.show(ctx, |ui| {
            //             self.render_stage(ui);
            //         });
            //     }
            // }
            // egui::Window::new("Please enter your password")
            //     .collapsible(false)
            //     .show(ctx, |ui| {
            //         self.render_stage(ui);
            //     });
            // disposition

            false
        }
    }

    fn render_stage(&mut self, ui: &mut Ui) {
        let stage = self.stages[self.index].clone();
        match stage(ui, &mut self.ctx.borrow_mut()) {
            Disposition::Previous => {
                if self.index > 0 {
                    self.index -= 1;
                }
            }
            Disposition::Next => {
                self.index += 1;
            }
            Disposition::Current => {}
        }
    }

    // pub fn with_secret(&mut self, callback: impl Fn(Secret) + Send + Sync + 'static) {
    //     self.callback = Some(Arc::new(callback));
    // }

    // pub fn cascade<FnRender, FnHandler, V>(
    //     &mut self,
    //     render: impl Fn(&mut Ui) -> Option<V> + Send + Sync + 'static,
    //     handler: impl Fn(V) + Send + Sync + 'static,
    // ) where
    //     FnRender: Fn(&mut Ui) -> Option<V> + Send + Sync + 'static,
    //     FnHandler: Fn(V) + Send + Sync + 'static,
    //     V: 'static,
    // {
    //     let binding = Binding { render, handler };

    //     let binding: Arc<dyn BindingT + Send + Sync + 'static> = Arc::new(binding);

    //     self.binding = Some(binding);
    // }

    // pub fn render(&mut self, ctx: &egui::Context) -> bool {
    //     if let Some(binding) = &self.binding.clone() {
    //         egui::Window::new("Please enter your password")
    //             .collapsible(false)
    //             .show(ctx, |ui| {
    //                 if binding.render(ui) {
    //                     self.binding = None
    //                 }
    //             });
    //         false
    //     } else if self.callback.is_some() {
    //         egui::Window::new("Please enter your password")
    //             .collapsible(false)
    //             .show(ctx, |ui| {
    //                 if let Some(secret) = self.render_secret_request(ui) {
    //                     (self.callback.take().unwrap())(secret.clone());
    //                 }
    //             });

    //         true
    //     } else {
    //         false
    //     }
    // }

    // fn render_secret_request(&mut self, ui: &mut Ui) -> Option<Secret> {
    //     let size = egui::Vec2::new(200_f32, 40_f32);

    //     let message = Some("Please enter you secret TEST:".to_string());
    //     if let Some(message) = &message {
    //         ui.label(" ");
    //         ui.label(egui::RichText::new(message).color(egui::Color32::from_rgb(255, 128, 128)));
    //         ui.label(" ");
    //     }

    //     ui.label(" ");
    //     ui.label(" ");

    //     ui.add_sized(
    //         size,
    //         egui::TextEdit::singleline(&mut self.secret)
    //             .hint_text("Enter Password...")
    //             .password(true)
    //             .vertical_align(egui::Align::Center),
    //     );

    //     // ui.add_sized(egui::Vec2::new(120_f32,40_f32), egui::Button::new("Testing 123"));

    //     if ui.add_sized(size, egui::Button::new("Unlock")).clicked() {
    //         println!("secret: {}", self.secret);
    //         let secret = kaspa_wallet_core::secret::Secret::new(self.secret.as_bytes().to_vec());
    //         self.secret.zeroize();
    //         Some(secret)
    //     } else {
    //         None
    //     }
    // }
}

impl<Ctx> BindingT for Wizard<Ctx> {
    fn render(&mut self, ctx: &egui::Context) -> bool {
        // fn render(&mut self, ctx: &egui::Context) -> bool {
        // let mut ctx = self.ctx.borrow_mut();
        self.render_impl(ctx)
    }
}

// pub fn with_secret(callback: impl Fn(Secret) + Send + Sync + 'static) {
//     prompt().with_secret(callback);
//     // self.callback = Some(Arc::new(callback));
// }

// // pub fn cascade<V>(
// pub fn cascade<FnRender, FnHandler, V>(
//     render: impl Fn(&mut Ui) -> Option<V> + Send + Sync + 'static,
//     handler: impl Fn(V) + Send + Sync + 'static,
// )
// where
//     FnRender: Fn(&mut Ui) -> Option<V> + Send + Sync + 'static,
//     FnHandler: Fn(V) + Send + Sync + 'static,
// V: 'static,
// {
//     prompt().cascade::<FnRender, FnHandler, V>(render, handler);
// }

// pub fn cascade<FnRender, FnHandler, V>(
//     render: impl Fn(&mut Ui) -> Option<V> + Send + Sync + 'static,
//     handler: impl Fn(V) + Send + Sync + 'static,
// ) where
//     FnRender: Fn(&mut Ui) -> Option<V> + Send + Sync + 'static,
//     FnHandler: Fn(V) + Send + Sync + 'static,
//     V: 'static,
// {
//     prompt().cascade::<FnRender, FnHandler, V>(render, handler);
// }

// - -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
// - =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// - -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
// - =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=

// static mut PROMPT: Option<Prompt> = None;

// pub fn prompt() -> &'static mut Prompt {
//     unsafe {
//         if PROMPT.is_none() {
//             PROMPT = Some(Prompt::new());
//         }
//         PROMPT.as_mut().unwrap()
//     }
// }
