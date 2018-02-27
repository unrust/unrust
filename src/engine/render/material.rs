use engine::asset::Asset;
use engine::render::{ShaderProgram, Texture};

use std::rc::Rc;
use std::collections::HashMap;
use na::{Vector2, Vector3, Vector4};

pub enum MaterialParam {
    Texture(Rc<Texture>),
    Float(f32),
    Vec2(Vector2<f32>),
    Vec3(Vector3<f32>),
    Vec4(Vector4<f32>),
}

impl From<f32> for MaterialParam {
    fn from(f: f32) -> MaterialParam {
        MaterialParam::Float(f)
    }
}
impl From<Rc<Texture>> for MaterialParam {
    fn from(f: Rc<Texture>) -> MaterialParam {
        MaterialParam::Texture(f)
    }
}
impl From<Vector2<f32>> for MaterialParam {
    fn from(f: Vector2<f32>) -> MaterialParam {
        MaterialParam::Vec2(f)
    }
}

impl From<Vector3<f32>> for MaterialParam {
    fn from(f: Vector3<f32>) -> MaterialParam {
        MaterialParam::Vec3(f)
    }
}

impl From<Vector4<f32>> for MaterialParam {
    fn from(f: Vector4<f32>) -> MaterialParam {
        MaterialParam::Vec4(f)
    }
}

pub struct Material {
    pub program: Rc<ShaderProgram>,
    pub params: HashMap<String, MaterialParam>,
}

impl Material {
    pub fn new(program: Rc<ShaderProgram>) -> Material {
        return Material {
            program: program,
            params: HashMap::new(),
        };
    }

    pub fn set<T>(&mut self, name: &str, t: T)
    where
        T: Into<MaterialParam>,
    {
        self.params.insert(name.to_string(), t.into());
    }
}

impl Asset for Material {
    type Resource = ();

    fn new_from_resource(_r: Self::Resource) -> Rc<Self> {
        unimplemented!();
    }
}
