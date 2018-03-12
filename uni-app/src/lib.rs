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
    pub show_cursor: bool,
}

impl AppConfig {
    pub fn new<T: Into<String>>(title: T, size: (u32, u32)) -> AppConfig {
        AppConfig {
            title: title.into(),
            size,
            vsync: true,
            show_cursor: true,
        }
    }
}

pub mod events {
    use std::fmt;

    #[derive(Debug, Clone)]
    pub struct ClickEvent;

    #[derive(Clone)]
    pub struct KeyDownEvent {
        pub code: String,
        pub shift: bool,
        pub alt: bool,
        pub ctrl: bool,
    }

    #[derive(Debug, Clone)]
    pub struct KeyPressEvent {
        pub code: String,
    }

    #[derive(Clone)]
    pub struct KeyUpEvent {
        pub code: String,
        pub shift: bool,
        pub alt: bool,
        pub ctrl: bool,
    }

    impl fmt::Debug for KeyUpEvent {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "{} {} {} {}",
                if self.shift { "shift" } else { "" },
                if self.alt { "alt" } else { "" },
                if self.ctrl { "ctrl" } else { "" },
                self.code,
            )
        }
    }

    impl fmt::Debug for KeyDownEvent {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "{} {} {} {}",
                if self.shift { "shift" } else { "" },
                if self.alt { "alt" } else { "" },
                if self.ctrl { "ctrl" } else { "" },
                self.code,
            )
        }
    }

}

pub use events::*;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Click(ClickEvent),
    KeyDown(KeyDownEvent),
    KeyUp(KeyUpEvent),
    Resized((u32, u32)),
    MousePos((f64, f64)),
}
