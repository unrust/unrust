#![feature(nll)]
#![recursion_limit = "512"]

// wasm-unknown-unknown
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

#[cfg(target_arch = "wasm32")]
#[path = "web_pad.rs"]
pub mod pad;

// NOT wasm-unknown-unknown
#[cfg(not(target_arch = "wasm32"))]
#[path = "native_pad.rs"]
pub mod pad;

pub use self::pad::*;
