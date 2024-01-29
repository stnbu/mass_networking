#[cfg(feature = "wasm")]
mod wasm;

mod aarch64;

#[cfg(feature = "wasm")]
pub use wasm::*;

pub use aarch64::*;
