use webgl::*;
use na::*;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use engine::core::{Component, ComponentBased, GameObject};
use engine::render::Camera;
use engine::render::{Directional, Light};
use engine::render::{Material, MaterialParam, Mesh, MeshBuffer, MeshSurface, ShaderProgram,
                     Texture};
use engine::asset::{AssetError, AssetSystem};

use super::imgui;

pub trait IEngine {
    fn new_gameobject(&mut self) -> Rc<RefCell<GameObject>>;

    fn asset_system<'a>(&'a self) -> &'a AssetSystem;

    fn asset_system_mut<'a>(&'a mut self) -> &'a mut AssetSystem;

    fn gui_context(&mut self) -> Rc<RefCell<imgui::Context>>;
}

pub struct Engine<A>
where
    A: AssetSystem,
{
    pub gl: WebGLRenderingContext,
    pub main_camera: Option<Rc<RefCell<Camera>>>,

    pub objects: Vec<Weak<RefCell<GameObject>>>,
    pub program_cache: RefCell<HashMap<&'static str, Rc<ShaderProgram>>>,
    pub asset_system: Box<A>,
    pub screen_size: (u32, u32),

    pub gui_context: Rc<RefCell<imgui::Context>>,
}

#[derive(Default)]
struct EngineContext {
    mesh_buffer: Weak<MeshBuffer>,
    prog: Weak<ShaderProgram>,
    textures: VecDeque<(u32, Weak<Texture>)>,

    main_light: Option<Arc<Component>>,
    point_lights: Vec<Arc<Component>>,

    switch_mesh: u32,
    switch_prog: u32,
    switch_tex: u32,
}

macro_rules! impl_cacher {
    ($k:ident, $t:ty) => {
        impl EngineCacher for $t {
            fn get_cache<'a>(ctx: &'a mut EngineContext) -> &'a mut Weak<Self> {
                &mut ctx.$k
            }
        }
    };
}

trait EngineCacher {
    fn get_cache(ctx: &mut EngineContext) -> &mut Weak<Self>;
}

impl_cacher!(prog, ShaderProgram);
impl_cacher!(mesh_buffer, MeshBuffer);

impl EngineContext {
    pub fn prepare_cache<T, F>(&mut self, new_p: &Rc<T>, bind: F) -> Result<(), AssetError>
    where
        T: EngineCacher,
        F: FnOnce(&mut EngineContext) -> Result<(), AssetError>,
    {
        if self.need_cache(new_p) {
            bind(self)?;
            *T::get_cache(self) = Rc::downgrade(new_p);
        }

        Ok(())
    }

    pub fn need_cache_tex(&self, new_tex: &Rc<Texture>) -> Option<u32> {
        for &(u, ref tex) in self.textures.iter() {
            if let Some(ref p) = tex.upgrade() {
                if Rc::ptr_eq(new_tex, p) {
                    return Some(u);
                }
            }
        }

        None
    }

    pub fn prepare_cache_tex<F>(
        &mut self,
        new_tex: &Rc<Texture>,
        bind: F,
    ) -> Result<u32, AssetError>
    where
        F: FnOnce(&mut EngineContext, u32) -> Result<(), AssetError>,
    {
        let found = self.need_cache_tex(new_tex);

        if found.is_none() {
            let mut unit = self.textures.len() as u32;

            // find the empty slots.
            if unit >= 8 {
                let opt_pos = self.textures
                    .iter()
                    .position(|&(_, ref t)| t.upgrade().is_none());

                unit = match opt_pos {
                    Some(pos) => self.textures.remove(pos).unwrap().0,
                    None => self.textures.pop_front().unwrap().0,
                }
            }

            return match bind(self, unit) {
                Ok(()) => {
                    self.textures.push_back((unit, Rc::downgrade(new_tex)));
                    Ok(unit)
                }
                Err(s) => Err(s),
            };
        }

        Ok(found.unwrap())
    }

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

struct RenderCommand {
    pub surface: Rc<MeshSurface>,
    pub model_m: Matrix4<f32>,
}

fn compute_model_m(object: &GameObject) -> Matrix4<f32> {
    // Setup Matrices
    let modelm = object.transform.to_homogeneous();
    modelm * Matrix4::new_nonuniform_scaling(&object.scale)
}

pub struct ClearOption {
    pub color: Option<(f32,f32,f32,f32)>,
    pub clear_color: bool,
    pub clear_depth: bool,
    pub clear_stencil: bool,
}

impl<A> Engine<A>
where
    A: AssetSystem,
{
    pub fn clear(&self, option: ClearOption) {
        if let Some(col) = option.color {
            self.gl.clear_color(col.0, col.1, col.2, col.3);
        } else {
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
        }
        if option.clear_color {
            self.gl.clear(BufferBit::Color);
        }
        if option.clear_depth {
            self.gl.clear(BufferBit::Depth);
        }
        if option.clear_stencil {
            self.gl.clear(BufferBit::Stencil);
        }
    }

    fn setup_material(
        &self,
        ctx: &mut EngineContext,
        material: &Material,
    ) -> Result<(), AssetError> {
        ctx.prepare_cache(&material.program, |ctx| {
            material.program.bind(&self.gl)?;
            ctx.switch_prog += 1;
            Ok(())
        })?;

        let prog = ctx.prog.upgrade().unwrap();

        for (name, param) in material.params.iter() {
            match param {
                &MaterialParam::Texture(ref tex) => {
                    let new_unit = ctx.prepare_cache_tex(&tex, |ctx, unit| {
                        // Binding texture
                        tex.bind(&self.gl, unit)?;

                        ctx.switch_tex += 1;
                        Ok(())
                    })?;

                    prog.set(&name, new_unit as i32);
                }

                &MaterialParam::Float(f) => {
                    prog.set(&name, f);
                }
                &MaterialParam::Vec2(v) => {
                    prog.set(&name, v);
                }
                &MaterialParam::Vec3(v) => {
                    prog.set(&name, v);
                }
                &MaterialParam::Vec4(v) => {
                    prog.set(&name, v);
                }
            }
        }

        self.setup_light(ctx, &prog);

        Ok(())
    }

    fn setup_camera(&self, ctx: &mut EngineContext, modelm: Matrix4<f32>, camera: &Camera) {
        let prog = ctx.prog.upgrade().unwrap();
        // setup_camera
        prog.set("uMVMatrix", camera.v * modelm);
        prog.set("uPMatrix", camera.p);
        prog.set("uNMatrix", modelm.try_inverse().unwrap().transpose());
        prog.set("uMMatrix", modelm);
        prog.set("uViewPos", camera.eye());
    }

    fn setup_light(&self, ctx: &EngineContext, prog: &ShaderProgram) {
        // Setup light

        let light_com = ctx.main_light.as_ref().unwrap();
        let light = light_com.try_as::<Light>().unwrap();
        light.borrow().bind("uDirectionalLight", &prog);

        for (i, plight_com) in ctx.point_lights.iter().enumerate() {
            let plight = plight_com.try_as::<Light>().unwrap();
            let name = format!("uPointLights[{}]", i);

            plight.borrow().bind(&name, &prog);
        }
    }

    fn render_commands(
        &self,
        ctx: &mut EngineContext,
        commands: Vec<RenderCommand>,
        camera: &Camera,
    ) {
        let gl = &self.gl;

        for cmd in commands {
            if let Err(err) = self.setup_material(ctx, &*cmd.surface.material) {
                if let AssetError::NotReady = err {
                    continue;
                }

                panic!(format!("Failed to load mesh, reason {:?}", err));
            }

            let prog = ctx.prog.upgrade().unwrap();

            let r = ctx.prepare_cache(&cmd.surface.buffer, |ctx| {
                cmd.surface.buffer.bind(&self.gl, &prog)?;
                ctx.switch_mesh += 1;
                Ok(())
            });

            self.setup_camera(ctx, cmd.model_m, camera);

            match r {
                Ok(_) => {
                    prog.commit(gl);
                    cmd.surface.buffer.render(gl);
                }
                Err(ref err) => match *err {
                    AssetError::NotReady => (),
                    _ => panic!(format!("Failed to load mesh, reason {:?}", err)),
                },
            }
        }
    }

    pub fn begin(&mut self) {
        imgui::begin();

        self.asset_system_mut().step();
    }

    pub fn end(&mut self) {}

    fn map_component<T, F>(&self, mut func: F)
    where
        T: 'static + ComponentBased,
        F: FnMut(Arc<Component>) -> bool,
    {
        for obj in self.objects.iter() {
            if let Some(r) = obj.upgrade().map_or(None, |obj| {
                obj.borrow().find_component::<T>().map(|(_, c)| c)
            }) {
                if !func(r) {
                    return;
                }
            }
        }
    }
    fn find_all_components<T>(&self) -> Vec<Arc<Component>>
    where
        T: 'static + ComponentBased,
    {
        let mut result = Vec::new();
        self.map_component::<T, _>(|c| {
            result.push(c);
            true
        });

        result
    }

    fn find_component<T>(&self) -> Option<Arc<Component>>
    where
        T: 'static + ComponentBased,
    {
        let mut r = None;
        self.map_component::<T, _>(|c| {
            r = Some(c);
            false
        });

        r
    }

    fn prepare_ctx(&self, ctx: &mut EngineContext) {
        // prepare main light.
        ctx.main_light = Some(self.find_component::<Light>().unwrap_or({
            Component::new(Light::Directional(Directional {
                direction: Vector3::new(0.5, -1.0, 1.0).normalize(),
                ambient: Vector3::new(0.2, 0.2, 0.2),
                diffuse: Vector3::new(0.5, 0.5, 0.5),
                specular: Vector3::new(1.0, 1.0, 1.0),
            }))
        }));

        ctx.point_lights = self.find_all_components::<Light>()
                .into_iter()
                .filter(|c| {
                    let light_com = c.try_as::<Light>().unwrap();
                    match *light_com.borrow() {
                        Light::Point(_) => true,
                        _ => false,
                    }
                })
                .take(4)            // only take 4 points light.
                .collect();
    }

    pub fn render_pass(&self, camera: &Camera, clear_option: ClearOption) {
        let objects = &self.objects;

        let mut ctx: EngineContext = Default::default();

        if let Some(ref rt) = camera.render_texture {
            rt.bind_frame_buffer(&self.gl);
        }

        if let Some(((x, y), (w, h))) = camera.rect {
            self.gl.viewport(x, y, w, h);
        } else {
            self.gl
                .viewport(0, 0, self.screen_size.0, self.screen_size.1);
        }

        self.clear(clear_option);

        self.prepare_ctx(&mut ctx);

        let mut commands = Vec::new();
        for obj in objects.iter() {
            obj.upgrade().map(|obj| {
                let object = obj.borrow();
                if object.active {
                    let result = object.find_component::<Mesh>();

                    if let Some((mesh, _)) = result {
                        for surface in mesh.surfaces.iter() {
                            commands.push(RenderCommand {
                                surface: surface.clone(),
                                model_m: compute_model_m(&*object),
                            })
                        }
                    }
                }
            });
        }

        self.render_commands(&mut ctx, commands, camera);

        if let Some(ref rt) = camera.render_texture {
            rt.unbind_frame_buffer(&self.gl);
        }
    }

    pub fn render(&mut self, clear_option: ClearOption) {
        imgui::pre_render(self);

        if let Some(ref camera) = self.main_camera.as_ref() {
            self.render_pass(&camera.borrow(), clear_option);
        }

        // drop all gameobjects if there are no other references
        self.objects.retain(|obj| obj.upgrade().is_some());
    }

    pub fn new(webgl_ctx: WebGLContext, size: (u32, u32)) -> Engine<A> {
        let gl = WebGLRenderingContext::new(webgl_ctx);

        /*=========Drawing the triangle===========*/

        // Clear the canvas
        gl.clear_color(0.5, 0.5, 0.5, 1.0);

        // Enable the depth test
        gl.enable(Flag::DepthTest as i32);

        // Enable alpha blending
        gl.enable(Flag::Blend as i32);

        gl.enable(Culling::CullFace as i32);
        gl.cull_face(Culling::Back);

        // Clear the color buffer bit
        gl.clear(BufferBit::Color);
        gl.clear(BufferBit::Depth);
        gl.blend_func(BlendMode::SrcAlpha, BlendMode::OneMinusSrcAlpha);

        // Set the view port
        gl.viewport(0, 0, size.0, size.1);

        Engine {
            gl: gl,
            main_camera: None,
            objects: vec![],
            program_cache: RefCell::new(HashMap::new()),
            asset_system: Box::new(A::new()),
            gui_context: Rc::new(RefCell::new(imgui::Context::new(size.0, size.1))),
            screen_size: size,
        }
    }
}

impl<A: AssetSystem> IEngine for Engine<A> {
    fn new_gameobject(&mut self) -> Rc<RefCell<GameObject>> {
        let go = Rc::new(RefCell::new(GameObject {
            transform: Isometry3::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            active: true,
            components: vec![],
        }));

        self.objects.push(Rc::downgrade(&go));
        go
    }

    fn gui_context(&mut self) -> Rc<RefCell<imgui::Context>> {
        self.gui_context.clone()
    }

    fn asset_system<'a>(&'a self) -> &'a AssetSystem {
        &*self.asset_system
    }

    fn asset_system_mut<'a>(&'a mut self) -> &'a mut AssetSystem {
        &mut *self.asset_system
    }
}
