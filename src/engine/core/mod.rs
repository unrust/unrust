mod game_object;
mod scene_tree;

pub use self::game_object::{Component, ComponentBased, GameObject};
pub use self::scene_tree::SceneTree;

pub mod internal {
    pub use super::game_object::GameObjectUtil;
}
