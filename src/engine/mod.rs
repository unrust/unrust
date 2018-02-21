mod core;
mod render;
mod asset;

pub mod imgui;
pub mod engine;

pub use self::imgui::Metric;

pub use self::render::*;
pub use self::asset::*;
pub use self::core::{Component, ComponentBased, GameObject};
pub use self::engine::IEngine;

pub type Engine<FS, F> = engine::Engine<AssetDatabase<FS, F>>;
