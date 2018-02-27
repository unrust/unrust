#![feature(nll)]
#![recursion_limit = "512"]

// wasm-unknown-unknown
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

#[cfg(target_arch = "wasm32")]
#[path = "web_app.rs"]
pub mod sys;

#[cfg(target_arch = "wasm32")]
#[path = "web_fs.rs"]
pub mod fs;

// NOT wasm-unknown-unknown
#[cfg(not(target_arch = "wasm32"))]
extern crate glutin;

#[cfg(not(target_arch = "wasm32"))]
extern crate time;

#[cfg(not(target_arch = "wasm32"))]
#[path = "native_app.rs"]
pub mod sys;

#[cfg(not(target_arch = "wasm32"))]
#[path = "native_fs.rs"]
pub mod fs;

pub use self::sys::*;
pub use self::fs::*;

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

mod events {
    #[derive(Debug, Clone)]
    pub struct ClickEvent;

    #[derive(Debug, Clone)]
    pub struct KeyDownEvent {
        pub code: String,
    }

    #[derive(Debug, Clone)]
    pub struct KeyPressEvent {
        pub code: String,
    }

    #[derive(Debug, Clone)]
    pub struct KeyUpEvent {
        pub code: String,
    }
}

pub use events::*;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Click(ClickEvent),
    KeyDown(KeyDownEvent),
    KeyUp(KeyUpEvent),
    Resized((u32, u32)),
}

pub struct FPS {
    counter: u32,
    last: f64,
    pub fps: u32,
}
