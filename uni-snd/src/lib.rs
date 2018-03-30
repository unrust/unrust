#![recursion_limit = "256"]

extern crate uni_app;

// wasm-unknown-unknown
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

#[cfg(target_arch = "wasm32")]
#[path = "web_snd.rs"]
pub mod snd;

// NOT wasm-unknown-unknown
#[cfg(not(target_arch = "wasm32"))]
extern crate cpal;

#[cfg(not(target_arch = "wasm32"))]
#[path = "native_snd.rs"]
pub mod snd;

pub use self::snd::*;

#[derive(Debug, Clone, Copy)]
pub enum SoundError {
    NoError,
    NoDevice,
    OutputStream,
    UnknownStreamFormat,
}

pub trait SoundGenerator<T>: Send {
    fn init(&mut self, sample_rate: f32);
    fn handle_event(&mut self, evt: T);
    fn next_value(&mut self) -> f32;
}
