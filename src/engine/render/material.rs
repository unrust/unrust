use engine::core::ComponentBased;
use engine::asset::Asset;
use engine::render::{ShaderProgram, Texture};

use std::rc::Rc;
use std::collections::HashMap;

pub enum MaterialParam {
    Texture(Rc<Texture>),
    Float(f32),
}

pub struct Material {
    pub program: Rc<ShaderProgram>,
    pub params: HashMap<String, MaterialParam>,
}

impl Material {
    pub fn new(program: Rc<ShaderProgram>, hm: HashMap<String, MaterialParam>) -> Material {
        return Material {
            program: program,
            params: hm,
        };
    }
}

impl Asset for Material {
    type Resource = ();

    fn new_from_resource(_r: Self::Resource) -> Rc<Self> {
        unimplemented!();
    }
}

impl ComponentBased for Material {}
