use engine::asset::{Asset, AssetResult};
use engine::render::{RenderQueue, ShaderProgram, Texture};

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use na::{Matrix4, Vector2, Vector3, Vector4};

pub enum MaterialParam {
    Texture(Rc<Texture>),
    Float(f32),
    Bool(bool),
    Vec2(Vector2<f32>),
    Vec3(Vector3<f32>),
    Vec4(Vector4<f32>),
    Matrix4(Matrix4<f32>),
}

macro_rules! impl_from_material_param {
    ($frm: ty, $to: ident) => {
        impl From<$frm> for MaterialParam {
            fn from(b: $frm) -> MaterialParam {
                MaterialParam::$to(b)
            }
        }
    };
}

impl_from_material_param!(bool, Bool);
impl_from_material_param!(f32, Float);
impl_from_material_param!(Rc<Texture>, Texture);
impl_from_material_param!(Vector2<f32>, Vec2);
impl_from_material_param!(Vector3<f32>, Vec3);
impl_from_material_param!(Vector4<f32>, Vec4);
impl_from_material_param!(Matrix4<f32>, Matrix4);

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CullMode {
    Off,
    Back,
    Front,
    FrontAndBack,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DepthTest {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

impl Default for DepthTest {
    fn default() -> DepthTest {
        DepthTest::Less
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct MaterialState {
    pub cull: Option<CullMode>,
    pub alpha_blending: Option<bool>,
    pub depth_write: Option<bool>,
    pub depth_test: Option<DepthTest>,
}

pub struct Material {
    pub program: Rc<ShaderProgram>,
    pub render_queue: RenderQueue,
    pub states: MaterialState,

    params: RefCell<HashMap<String, MaterialParam>>,
}

impl Material {
    pub fn new(program: Rc<ShaderProgram>) -> Material {
        return Material {
            render_queue: RenderQueue::Opaque,
            program: program,
            params: RefCell::new(HashMap::new()),
            states: MaterialState::default(),
        };
    }

    pub fn set<T>(&self, name: &str, t: T)
    where
        T: Into<MaterialParam>,
    {
        self.params.borrow_mut().insert(name.to_string(), t.into());
    }

    pub fn bind<F>(&self, mut request_tex_unit: F) -> AssetResult<()>
    where
        F: FnMut(&Rc<Texture>) -> AssetResult<u32>,
    {
        for (name, param) in self.params.borrow().iter() {
            match param {
                &MaterialParam::Texture(ref tex) => {
                    let new_unit = request_tex_unit(&tex)?;
                    self.program.set(&name, (Rc::downgrade(&tex), new_unit));
                }
                &MaterialParam::Bool(v) => {
                    self.program.set(&name, v);
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
