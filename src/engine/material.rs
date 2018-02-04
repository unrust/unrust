use ComponentBased;
use Component;
use std::sync::Arc;
use engine::texture::Texture;

pub struct Material {
    pub program: &'static str,
    pub texture: Texture,
}

impl Material {
    pub fn new(s: &'static str, texture: Texture) -> Material {
        return Material {
            program: s,
            texture: texture,
        };
    }

    pub fn new_component(s: &'static str, texture: Texture) -> Arc<Component> {
        Component::new(Material::new(s, texture))
    }
}

impl ComponentBased for Material {}
