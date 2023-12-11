#![warn(clippy::all, rust_2018_idioms)]

extern crate self as kaspa_ng_core;

mod core;
pub use core::Core;

pub mod adaptor;
pub mod app;
pub mod collection;
pub mod device;
pub mod egui;
pub mod error;
pub mod events;
pub mod fonts;
pub mod imports;
pub mod market;
pub mod menu;
pub mod mobile;
pub mod modules;
pub mod network;
pub mod notifications;
pub mod primitives;
pub mod result;
pub mod runtime;
pub mod settings;
pub mod state;
pub mod status;
pub mod sync;
pub mod utils;
