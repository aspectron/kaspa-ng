#![warn(clippy::all, rust_2018_idioms)]

mod wallet;
pub use wallet::Wallet;

pub mod egui;
pub mod error;
pub mod events;
pub mod icons;
pub mod imports;
pub mod interop;
pub mod network;
pub mod panel;
pub mod primitives;
pub mod prompt;
pub mod result;
pub mod section;
pub mod settings;
pub mod sync;
pub mod theme;
