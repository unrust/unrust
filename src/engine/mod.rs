mod core;
mod render;
mod asset;

pub mod imgui;
pub mod context;
pub mod engine;

pub use self::imgui::Metric;

pub use self::render::*;
pub use self::asset::*;
pub use self::core::{Component, ComponentBased, ComponentEvent, GameObject, SceneTree};
pub use self::core::Aabb;

pub use self::engine::{ClearOption, IEngine};

pub type Engine<FS, F> = engine::Engine<AssetDatabase<FS, F>>;
