#![warn(clippy::all, rust_2018_idioms)]

mod wallet;
pub use wallet::Wallet;

pub mod error;
pub mod events;
pub mod imports;
pub mod interop;
pub mod network;
pub mod primitives;
pub mod prompt;
pub mod result;
pub mod section;
// pub mod sequence;
pub mod settings;
pub mod sync;
pub mod wizard;
