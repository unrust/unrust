use ComponentBased;
use Component;
use std::sync::Arc;
use std::rc::Rc;
use engine::texture::Texture;

pub struct Material {
    pub program: &'static str,
    pub texture: Rc<Texture>,
}

impl Material {
    pub fn new(s: &'static str, texture: Rc<Texture>) -> Material {
        return Material {
            program: s,
            texture: texture,
        };
    }

    pub fn new_component(s: &'static str, texture: Rc<Texture>) -> Arc<Component> {
        Component::new(Material::new(s, texture))
    }
}

impl ComponentBased for Material {}
