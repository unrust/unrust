use engine::asset::{Asset, AssetResult};
use engine::render::{RenderQueue, ShaderProgram, Texture};

use fnv::FnvHashMap;
use math::*;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct TexturePtr(Rc<Texture>);

impl PartialEq for TexturePtr {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MaterialParam {
    Texture(TexturePtr),
    Float(f32),
    Bool(bool),
    Vec2(Vector2<f32>),
    Vec3(Vector3<f32>),
    Vec4(Vector4<f32>),
    Matrix4(Matrix4<f32>),
    Params(MaterialParamMap),
}

pub type MaterialParamMap = FnvHashMap<Cow<'static, str>, MaterialParam>;

macro_rules! impl_from_material_param {
    ($frm:ty, $to:ident) => {
        impl From<$frm> for MaterialParam {
            fn from(b: $frm) -> MaterialParam {
                MaterialParam::$to(b)
            }
        }
    };
}

impl_from_material_param!(bool, Bool);
impl_from_material_param!(f32, Float);
impl_from_material_param!(Vector2<f32>, Vec2);
impl_from_material_param!(Vector3<f32>, Vec3);
impl_from_material_param!(Vector4<f32>, Vec4);
impl_from_material_param!(Matrix4<f32>, Matrix4);
impl_from_material_param!(MaterialParamMap, Params);

impl From<Rc<Texture>> for MaterialParam {
    fn from(b: Rc<Texture>) -> MaterialParam {
        MaterialParam::Texture(TexturePtr(b))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CullMode {
    Off,
    Back,
    Front,
    FrontAndBack,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct MaterialState {
    pub cull: Option<CullMode>,
    pub alpha_blending: Option<bool>,
    pub depth_write: Option<bool>,
    pub depth_test: Option<DepthTest>,
}

#[derive(Debug)]
pub struct Material {
    pub program: Rc<ShaderProgram>,
    pub render_queue: RenderQueue,
    pub states: MaterialState,

    params: RefCell<MaterialParamMap>,
}

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.program, &other.program) && self.render_queue == other.render_queue
            && self.states == other.states
            && *self.params.borrow() == *other.params.borrow()
    }
}

impl Material {
    pub fn new(program: Rc<ShaderProgram>) -> Material {
        return Material {
            render_queue: RenderQueue::Opaque,
            program: program,
            params: RefCell::new(FnvHashMap::default()),
            states: MaterialState::default(),
        };
    }

    pub fn set<T, S>(&self, name: S, t: T)
    where
        T: Into<MaterialParam>,
        S: Into<Cow<'static, str>>,
    {
        self.params.borrow_mut().insert(name.into(), t.into());
    }

    fn bind_params<F>(
        &self,
        params: &MaterialParamMap,
        request_tex_unit: &mut F,
        level: u32,
    ) -> AssetResult<()>
    where
        F: FnMut(&Rc<Texture>) -> AssetResult<u32>,
    {
        for (name, param) in params.iter() {
            match param {
                &MaterialParam::Texture(ref tex) => {
                    let new_unit = request_tex_unit(&tex.0)?;
                    self.program
                        .set(name.clone(), (Rc::downgrade(&tex.0), new_unit));
                }
                &MaterialParam::Bool(v) => {
                    self.program.set(name.clone(), v);
                }
                &MaterialParam::Float(f) => {
                    self.program.set(name.clone(), f);
                }
                &MaterialParam::Vec2(v) => {
                    self.program.set(name.clone(), v);
                }
                &MaterialParam::Vec3(v) => {
                    self.program.set(name.clone(), v);
                }
                &MaterialParam::Vec4(v) => {
                    self.program.set(name.clone(), v);
                }
                &MaterialParam::Matrix4(v) => {
                    self.program.set(name.clone(), v);
                }
                &MaterialParam::Params(ref pm) => {
                    self.bind_params(&pm, request_tex_unit, level + 1)?;
                }
            }
        }

        Ok(())
    }

    pub fn bind<F>(&self, mut request_tex_unit: F) -> AssetResult<()>
    where
        F: FnMut(&Rc<Texture>) -> AssetResult<u32>,
    {
        self.bind_params(&self.params.borrow(), &mut request_tex_unit, 0)?;

        Ok(())
    }
}

impl Asset for Material {
    type Resource = ();

    fn new_from_resource(_r: Self::Resource) -> Rc<Self> {
        unimplemented!();
    }
}
