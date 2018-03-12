use engine::asset::{Asset, AssetResult};
use engine::render::{RenderQueue, ShaderProgram, Texture};

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use na::{Matrix4, Vector2, Vector3, Vector4};

pub enum MaterialParam {
    Texture(Rc<Texture>),
    Float(f32),
    Vec2(Vector2<f32>),
    Vec3(Vector3<f32>),
    Vec4(Vector4<f32>),
    Matrix4(Matrix4<f32>),
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

impl From<Matrix4<f32>> for MaterialParam {
    fn from(f: Matrix4<f32>) -> MaterialParam {
        MaterialParam::Matrix4(f)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CullMode {
    Off,
    Back,
    Front,
    FrontAndBack,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MaterialState {
    Cull(CullMode),
}

pub struct Material {
    pub program: Rc<ShaderProgram>,
    pub render_queue: RenderQueue,
    pub states: Vec<MaterialState>,

    params: RefCell<HashMap<String, MaterialParam>>,
}

impl Material {
    pub fn new(program: Rc<ShaderProgram>) -> Material {
        return Material {
            render_queue: RenderQueue::Opaque,
            program: program,
            params: RefCell::new(HashMap::new()),
            states: Vec::new(),
        };
    }

    pub fn set<T>(&self, name: &str, t: T)
    where
        T: Into<MaterialParam>,
    {
        self.params.borrow_mut().insert(name.to_string(), t.into());
    }

    pub fn bind<F>(&self, mut f: F) -> AssetResult<()>
    where
        F: FnMut(&Rc<Texture>) -> AssetResult<u32>,
    {
        for (name, param) in self.params.borrow().iter() {
            match param {
                &MaterialParam::Texture(ref tex) => {
                    let new_unit = f(&tex)?;
                    self.program.set(&name, new_unit as i32);
                }

                &MaterialParam::Float(f) => {
                    self.program.set(&name, f);
                }
                &MaterialParam::Vec2(v) => {
                    self.program.set(&name, v);
                }
                &MaterialParam::Vec3(v) => {
                    self.program.set(&name, v);
                }
                &MaterialParam::Vec4(v) => {
                    self.program.set(&name, v);
                }
                &MaterialParam::Matrix4(v) => {
                    self.program.set(&name, v);
                }
            }
        }

        Ok(())
    }
}

impl Asset for Material {
    type Resource = ();

    fn new_from_resource(_r: Self::Resource) -> Rc<Self> {
        unimplemented!();
    }
}
