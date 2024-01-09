//! Experimental markup language
//! <https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/easy_mark/mod.rs>

pub mod easy_mark_parser;
mod easy_mark_viewer;

// pub use easy_mark_parser as parser;
pub use easy_mark_viewer::easy_mark;
