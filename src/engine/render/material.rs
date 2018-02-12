use engine::core::ComponentBased;
use engine::Asset;
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

impl Asset for Material {
    fn new(_s: &str) -> Rc<Self> {
        unimplemented!();
    }
}

impl ComponentBased for Material {}
