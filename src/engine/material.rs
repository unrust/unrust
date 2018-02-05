use ComponentBased;
use Component;
use std::sync::Arc;
use std::rc::Rc;
use engine::texture::Texture;
use ShaderProgram;

pub struct Material {
    pub program: Rc<ShaderProgram>,
    pub texture: Rc<Texture>,
}

impl Material {
    pub fn new(program: Rc<ShaderProgram>, texture: Rc<Texture>) -> Material {
        return Material {
            program: program,
            texture: texture,
        };
    }

    pub fn new_component(program: Rc<ShaderProgram>, texture: Rc<Texture>) -> Arc<Component> {
        Component::new(Material::new(program, texture))
    }
}

impl ComponentBased for Material {}
