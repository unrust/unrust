use std::rc::{Rc, Weak};
use std::sync::Arc;
use engine::render::{CullMode, DepthTest, Material, MaterialState, MeshBuffer, ShaderProgram,
                     Texture};
use std::collections::VecDeque;
use engine::core::Component;
use engine::engine::EngineStats;
use webgl::{Culling, WebGLRenderingContext};
use webgl;
use engine::asset::AssetResult;

trait ToGLState<T> {
    fn as_gl_state(&self) -> T;
}

impl ToGLState<webgl::DepthTest> for DepthTest {
    fn as_gl_state(&self) -> webgl::DepthTest {
        match self {
            &DepthTest::Never => webgl::DepthTest::Never,
            &DepthTest::Always => webgl::DepthTest::Always,
            &DepthTest::Less => webgl::DepthTest::Less,
            &DepthTest::LessEqual => webgl::DepthTest::Lequal,
            &DepthTest::Greater => webgl::DepthTest::Greater,
            &DepthTest::NotEqual => webgl::DepthTest::Notequal,
            &DepthTest::GreaterEqual => webgl::DepthTest::Gequal,
            &DepthTest::Equal => webgl::DepthTest::Equal,
        }
    }
}

#[derive(Default)]
pub struct StateCache {
    state: MaterialState,
    curr: MaterialState,
}

impl StateCache {
    pub fn apply_defaults(&mut self) {
        self.curr = MaterialState {
            cull: Some(CullMode::Back),
            depth_test: Some(DepthTest::Less),
            depth_write: Some(true),
        }
    }

    pub fn apply(&mut self, ms: &MaterialState) {
        ms.cull.map(|s| self.curr.cull = Some(s));
        ms.depth_test.map(|s| self.curr.depth_test = Some(s));
        ms.depth_write.map(|s| self.curr.depth_write = Some(s));
    }

    pub fn commit(&mut self, gl: &WebGLRenderingContext) {
        self.curr.cull.map(|s| self.apply_cull(gl, &s));
        self.curr.depth_test.map(|s| self.apply_depth_test(gl, &s));
        self.curr.depth_write.map(|s| self.apply_depth_write(gl, s));
    }

    fn apply_depth_write(&mut self, gl: &WebGLRenderingContext, b: bool) {
        if let Some(curr_b) = self.state.depth_write {
            if curr_b == b {
                return;
            }
        }

        gl.depth_mask(b);
        self.state.depth_write = Some(b);
    }

    fn apply_depth_test(&mut self, gl: &WebGLRenderingContext, ct: &DepthTest) {
        if let Some(s) = self.state.depth_test {
            if s == *ct {
                return;
            }
        }

        if let &DepthTest::Never = ct {
            gl.disable(webgl::Flag::DepthTest as i32);
        } else {
            gl.enable(webgl::Flag::DepthTest as i32);
            gl.depth_func(ct.as_gl_state());
        }

        self.state.depth_test = Some(*ct);
    }

    fn apply_cull(&mut self, gl: &WebGLRenderingContext, cm: &CullMode) {
        if let Some(s) = self.state.cull {
            if s == *cm {
                return;
            }
        }

        match cm {
            &CullMode::Off => {
                gl.disable(Culling::CullFace as i32);
            }
            &CullMode::Front => {
                gl.enable(Culling::CullFace as i32);
                gl.cull_face(Culling::Front);
            }
            &CullMode::Back => {
                gl.enable(Culling::CullFace as i32);
                gl.cull_face(Culling::Back);
            }
            &CullMode::FrontAndBack => {
                gl.enable(Culling::CullFace as i32);
                gl.cull_face(Culling::FrontAndBack);
            }
        }

        self.state.cull = Some(*cm);
    }
}

pub struct EngineContext {
    pub mesh_buffer: Weak<MeshBuffer>,
    pub prog: Weak<ShaderProgram>,
    pub textures: VecDeque<(u32, Weak<Texture>)>,

    pub main_light: Option<Arc<Component>>,
    pub point_lights: Vec<Arc<Component>>,

    pub switch_mesh: u32,
    pub switch_prog: u32,
    pub switch_tex: u32,

    pub stats: EngineStats,
    pub states: StateCache,

    pub last_light_bound: Option<Weak<ShaderProgram>>,
    pub last_material_bound: Option<Weak<Material>>,
}

impl EngineContext {
    pub fn new(stats: EngineStats) -> EngineContext {
        EngineContext {
            mesh_buffer: Default::default(),
            prog: Default::default(),
            textures: Default::default(),

            main_light: Default::default(),
            point_lights: Default::default(),

            switch_mesh: 0,
            switch_prog: 0,
            switch_tex: 0,

            stats: stats,

            states: Default::default(),
            last_light_bound: None,
            last_material_bound: None,
        }
    }
}

macro_rules! impl_cacher {
    ($k: ident, $t: ty) => {
        impl EngineCacher for $t {
            fn get_cache(ctx: &mut EngineContext) -> &mut Weak<Self> {
                &mut ctx.$k
            }
        }
    };
}

pub trait EngineCacher {
    fn get_cache(ctx: &mut EngineContext) -> &mut Weak<Self>;
}

impl_cacher!(prog, ShaderProgram);
impl_cacher!(mesh_buffer, MeshBuffer);

const MAX_TEXTURE_UNITS: u32 = 8;

impl EngineContext {
    #[cfg_attr(feature = "flame_it", flame)]
    pub fn prepare_cache<T, F>(&mut self, new_p: &Rc<T>, bind: F) -> AssetResult<()>
    where
        T: EngineCacher,
        F: FnOnce(&mut EngineContext) -> AssetResult<()>,
    {
        if self.need_cache(new_p) {
            bind(self)?;
            *T::get_cache(self) = Rc::downgrade(new_p);
        }

        Ok(())
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn need_cache_tex(&self, new_tex: &Rc<Texture>) -> Option<(usize, u32)> {
        for (pos, &(u, ref tex)) in self.textures.iter().enumerate() {
            if let Some(ref p) = tex.upgrade() {
                if Rc::ptr_eq(new_tex, p) {
                    return Some((pos, u));
                }
            }
        }

        None
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn prepare_cache_tex<F>(&mut self, new_tex: &Rc<Texture>, bind_func: F) -> AssetResult<u32>
    where
        F: FnOnce(&mut EngineContext, u32) -> AssetResult<()>,
    {
        if let Some((pos, unit)) = self.need_cache_tex(new_tex) {
            // move the used unit pos to the back
            self.textures.remove(pos);
            self.textures.push_back((unit, Rc::downgrade(&new_tex)));

            return Ok(unit);
        }

        let mut unit = self.textures.len() as u32;

        // find the empty slots.
        if unit >= MAX_TEXTURE_UNITS {
            let opt_pos = self.textures
                .iter()
                .position(|&(_, ref t)| t.upgrade().is_none());

            unit = match opt_pos {
                Some(pos) => self.textures.remove(pos).unwrap().0,
                None => self.textures.pop_front().unwrap().0,
            }
        }

        bind_func(self, unit).map(|_| {
            self.textures.push_back((unit, Rc::downgrade(new_tex)));
            unit
        })
    }

    #[cfg_attr(feature = "flame_it", flame)]
    fn need_cache<T>(&mut self, new_p: &Rc<T>) -> bool
    where
        T: EngineCacher,
    {
        match T::get_cache(self).upgrade() {
            None => true,
            Some(ref p) => !Rc::ptr_eq(new_p, p),
        }
    }
}
