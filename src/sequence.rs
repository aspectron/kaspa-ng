use std::{collections::VecDeque, marker::PhantomData};

use crate::imports::*;
pub use futures::{future::FutureExt, select, Future};

// trait StageT: 'static {
trait StageT {
    fn render(&self, ui: &mut Ui) -> bool;
}

// pub struct StageBinding<FnRender, In, Out>
// where
//     FnRender: Fn(&mut Ui, In) -> Option<Out> + 'static,
//     // FnHandler: Fn(V) + Send + Sync + 'static,
// {
//     render: FnRender,
// }

// impl<FnRender, In, Out> StageT for StageBinding<FnRender, In, Out>
// where
//     FnRender: Fn(&mut Ui, In) -> Option<Out> + 'static,
//     // FnHandler: Fn(V) + Send + Sync + 'static,
//     // I: 'static,
// {
//     fn render(&self, ui: &mut Ui) -> bool {
//         false
//         // if let Some(resp) = (self.render)(ui) {
//         //     (self.handler)(resp);
//         //     true
//         // } else {
//         //     false
//         // }
//     }

//     // fn then(self, render: FnRender) -> Stage<FnRender, V>
//     // where
//     //     FnRender: Fn(&mut Ui) -> Option<V> + Send + Sync + 'static,
//     //     V: 'static,
//     // {
//     //     Stage { render }
//     // }
// }

// pub struct Stage//<FnRender, V>
// // where
// //     FnRender: Fn(&mut Ui) -> Option<V> + Send + Sync + 'static,
// {
//     // render: FnRender,
//     binding: Rc<RefCell<VecDeque<Box<dyn StageT>>>>,

// }

// impl Stage {
//     pub fn stage<FnRender, Out>(
//         self : &Rc<Self>,
//         // &mut self,
//         render: impl Fn(&mut Ui) -> Option<Out> + 'static,
//     ) -> NextStage<Out> where
//         FnRender: Fn(&mut Ui) -> Option<Out> + 'static,
//         // Out: 'static,
//     {
//         let binding = Binding { render };

//         let binding: Box<dyn StageT> = Box::new(binding);

//         // self.binding = Some(binding);
//         self.binding.borrow_mut().push_back(binding);

//         NextStage::new(self)
//     }

// }

// pub struct NextStage<V>//<FnRender, V>
// // where
// //     FnRender: Fn(&mut Ui) -> Option<V> + Send + Sync + 'static,
// {
//     // render: FnRender,
//     // binding: Arc<dyn StageT + Send + Sync>,
//     _phantom: PhantomData<V>,
//     stage : Rc<Stage>,
// }

// impl<V> NextStage<V> {
//     pub fn new(stage : &Rc<Stage>) -> Self {
//         Self {
//             // binding: Arc::new(binding),
//             _phantom: PhantomData,
//             stage : stage.clone(),
//         }
//     }

//     pub fn then<FnRender>(
//         // self : &Rc<Self>,
//         &mut self,
//         render: impl Fn(&mut Ui) -> Option<V> + 'static,
//     ) -> NextStage<V> where
//         FnRender: Fn(&mut Ui) -> Option<V> + 'static,
//         V: 'static,
//     {
//         let binding = Binding { render };

//         let binding: Box<dyn StageT> = Box::new(binding);

//         // self.binding = Some(binding);
//         self.stage.binding.borrow_mut().push_back(binding);

//         NextStage::new(&self.stage)
//     }

//     pub fn finish<FnRender>(
//         // self : &Rc<Self>,
//         &mut self,
//         finish: impl Fn(&mut Ui) -> Option<V> + 'static,
//     ) -> NextStage<V> where
//         FnRender: Fn(&mut Ui) -> Option<V> + 'static,
//         V: 'static,
//     {
//         let binding = Binding { render };

//         let binding: Box<dyn StageT> = Box::new(binding);

//         // self.binding = Some(binding);
//         self.stage.binding.borrow_mut().push_back(binding);

//         NextStage::new(&self.stage)
//     }

// }

//     pub fn stage<FnRender, FnHandler, V>(
//         &mut self,
//         render: impl Fn(&mut Ui) -> Option<V> + Send + Sync + 'static,
//     ) -> Stage where
//         FnRender: Fn(&mut Ui) -> Option<V> + Send + Sync + 'static,
//         V: 'static,
//     {
//         let binding = Binding { render };

//         let binding: Arc<dyn StageT + Send + Sync + 'static> = Arc::new(binding);

//         self.binding = Some(binding);
//     }

// }
