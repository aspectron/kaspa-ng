#![warn(clippy::all, rust_2018_idioms)]

mod wallet;
pub use wallet::Wallet;

pub mod error;
pub mod events;
pub mod imports;
pub mod interop;
pub mod primitives;
pub mod result;
pub mod secret;
pub mod section;
pub mod settings;
pub mod sync;
