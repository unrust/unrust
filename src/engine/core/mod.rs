mod game_object;
mod scene_tree;
mod math;

pub use self::game_object::{Component, ComponentBased, GameObject, IntoComponentPtr};
pub use self::scene_tree::{ComponentEvent, SceneTree};
pub use self::math::*;

pub mod internal {
    pub use super::game_object::GameObjectUtil;
}
