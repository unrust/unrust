mod asset;
mod core;
mod render;

pub mod context;
pub mod engine;
pub mod imgui;
pub mod sound;

pub use self::imgui::Metric;

pub use self::asset::*;
pub use self::core::Aabb;
pub use self::core::{Component, ComponentArena, ComponentBased, ComponentEvent, ComponentType,
                     GameObject, IntoComponentPtr, SceneTree};
pub use self::render::*;

pub use self::engine::{ClearOption, IEngine};

pub use self::sound::{SoundHandle, SoundSystem};

pub type Engine<FS, F> = engine::Engine<AssetDatabase<FS, F>>;
