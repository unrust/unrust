#![feature(nll)]
#![recursion_limit = "512"]

// wasm-unknown-unknown
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

#[cfg(target_arch = "wasm32")]
pub mod web_app;

#[cfg(target_arch = "wasm32")]
pub use self::web_app::*;

// NOT wasm-unknown-unknown
#[cfg(not(target_arch = "wasm32"))]
extern crate glutin;

#[cfg(not(target_arch = "wasm32"))]
pub mod native_app;

#[cfg(not(target_arch = "wasm32"))]
pub use self::native_app::*;

pub struct AppConfig {
    pub title: String,
    pub size: (u32, u32),
    pub vsync: bool,
}

impl AppConfig {
    pub fn new<T: Into<String>>(title: T, size: (u32, u32)) -> AppConfig {
        AppConfig {
            title: title.into(),
            size,
            vsync: true,
        }
    }
}

pub enum AppEvent {
    Click,
}
