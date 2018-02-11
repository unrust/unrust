use engine::core::ComponentBased;
use super::{ShaderProgram, Texture};

use std::rc::Rc;

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
}

impl ComponentBased for Material {}
