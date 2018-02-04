use ComponentBased;
use Component;
use std::sync::Arc;

pub struct Material {
    pub program: &'static str,
}

impl Material {
    pub fn new(s: &'static str) -> Material {
        return Material { program: s };
    }

    pub fn new_component(s: &'static str) -> Arc<Component> {
        Component::new(Material::new(s))
    }
}

impl ComponentBased for Material {}
