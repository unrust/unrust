mod component_arena;
mod game_object;
mod math;
mod scene_tree;

pub use self::component_arena::ComponentArena;
pub use self::game_object::{Component, ComponentBased, ComponentType, GameObject, IntoComponentPtr};
pub use self::math::*;
pub use self::scene_tree::{ComponentEvent, SceneTree};

pub mod internal {
    pub use super::game_object::GameObjectUtil;
}
