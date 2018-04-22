use math::*;
use webgl::*;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use engine::asset::{AssetError, AssetResult, AssetSystem};
use engine::context::EngineContext;
use engine::core::{Component, ComponentArena, ComponentBased, GameObject, SceneTree};
use engine::render::Camera;
use engine::render::{DepthTest, DirectionalLight, Light, Material, MaterialState, Mesh,
                     MeshSurface, ShaderProgram};
use engine::render::{Frustum, RenderQueue};
use image;
use math::Aabb;

use std::default::Default;

use super::imgui;

pub trait IEngine {
    fn new_game_object(&mut self, parent: &GameObject) -> Rc<RefCell<GameObject>>;

    fn asset_system<'a>(&'a self) -> &'a AssetSystem;

    fn asset_system_mut<'a>(&'a mut self) -> &'a mut AssetSystem;

    fn gui_context(&mut self) -> Rc<RefCell<imgui::Context>>;

    fn screen_size(&self) -> (u32, u32);

    fn hidpi_factor(&self) -> f32;
}

#[derive(Default, Copy, Clone)]
pub struct EngineStats {
    pub surfaces_count: u32,
    pub opaque_count: u32,
    pub transparent_count: u32,
    pub total_opaque_count: u32,
    pub total_transparent_count: u32,
}

pub struct Engine<A>
where
    A: AssetSystem,
{
    pub gl: WebGLRenderingContext,
    pub objects: Vec<Weak<RefCell<GameObject>>>,
    pub program_cache: RefCell<HashMap<&'static str, Rc<ShaderProgram>>>,
    pub asset_system: Box<A>,
    pub screen_size: (u32, u32),
    pub hidpi: f32,
    pub current_camera: RefCell<Option<Arc<Component>>>,
    pub gui_context: Rc<RefCell<imgui::Context>>,
    pub arena: Rc<ComponentArena>,

    pub stats: EngineStats,
}

struct RenderCommand {
    pub surface: Rc<MeshSurface>,
    pub model_m: Matrix4<f32>,
    pub cam_distance: f32,
}

#[derive(Default)]
struct RenderQueueState {
    states: MaterialState,
    commands: Vec<RenderCommand>,
}

impl RenderQueueState {
    fn sort_by_cam_distance(&mut self) -> &mut Self {
        self.commands.sort_unstable_by(|a, b| {
            let adist: f32 = a.cam_distance;
            let bdist: f32 = b.cam_distance;

            bdist.partial_cmp(&adist).unwrap()
        });

        self
    }

    fn sort_by_cam_distance_reverse(&mut self) -> &mut Self {
        self.commands.sort_by(|a, b| {
            let adist: f32 = a.cam_distance;
            let bdist: f32 = b.cam_distance;

            adist.partial_cmp(&bdist).unwrap()
        });

        self
    }

    fn sort_by_material(&mut self) -> &mut Self {
        self.commands.sort_by(|a, b| {
            let prog_a: &Material = &a.surface.material;
            let prog_b: &Material = &b.surface.material;

            let adist = prog_a as *const Material;
            let bdist = prog_b as *const Material;

            adist.partial_cmp(&bdist).unwrap()
        });

        self
    }
}

#[derive(Default)]
struct RenderQueueList {
    aabb: Option<Aabb>,
    queues: BTreeMap<RenderQueue, RenderQueueState>,
}

impl RenderQueueList {
    pub fn new() -> RenderQueueList {
        let mut qlist = RenderQueueList::default();

        // Opaque Queue
        let mut state = RenderQueueState::default();
        state.states.alpha_blending = Some(false);
        qlist.queues.insert(RenderQueue::Opaque, state);

        // Skybox Queue
        let mut state = RenderQueueState::default();
        state.states.depth_write = Some(false);
        state.states.alpha_blending = Some(false);
        state.states.depth_test = Some(DepthTest::LessEqual);
        qlist.queues.insert(RenderQueue::Skybox, state);

        // Transparent Queue
        let mut state = RenderQueueState::default();
        state.states.alpha_blending = Some(true);
        state.states.depth_write = Some(false);
        qlist.queues.insert(RenderQueue::Transparent, state);

        // UI Queue
        let mut state = RenderQueueState::default();
        state.states.alpha_blending = Some(true);
        qlist.queues.insert(RenderQueue::UI, state);

        qlist
    }

    fn surface_count(&self) -> usize {
        let mut n = 0;
        for (_, q) in self.queues.iter() {
            n += q.commands.len();
        }
        n
    }
}

fn compute_model_m(object: &GameObject) -> Matrix4<f32> {
    object.transform.as_global_matrix()
}

pub struct ClearOption {
    pub color: Option<(f32, f32, f32, f32)>,
    pub clear_color: bool,
    pub clear_depth: bool,
    pub clear_stencil: bool,
}

impl Default for ClearOption {
    fn default() -> Self {
        ClearOption {
            color: Some((0.3, 0.3, 0.3, 1.0)),
            clear_color: true,
            clear_depth: true,
            clear_stencil: false,
        }
    }
}

fn get_max_scale(s: &Vector3<f32>) -> f32 {
    s[0].max(s[1]).max(s[2])
}

impl<A> Engine<A>
where
    A: AssetSystem,
{
    pub fn new_scene_tree(&self) -> Rc<SceneTree> {
        SceneTree::new()
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn clear(&self, option: ClearOption) {
        if let Some(col) = option.color {
            self.gl.clear_color(col.0, col.1, col.2, col.3);
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

    pub fn resize(&mut self, size: (u32, u32)) {
        self.screen_size = size;

        self.gui_context.borrow_mut().reset();
    }

    #[cfg_attr(feature = "flame_it", flame)]
    fn setup_material(&self, ctx: &mut EngineContext, material: &Rc<Material>) -> AssetResult<()> {
        if let Some(ref last_material) = ctx.last_material_bound {
            if let Some(last_material) = last_material.upgrade() {
                if Rc::ptr_eq(&last_material, &material) {
                    return Ok(());
                }
            }
        }

        ctx.prepare_cache(&material.program, |ctx| {
            material.program.bind(&self.gl)?;
            ctx.switch_prog += 1;
            Ok(())
        })?;

        material.bind(|tex| {
            ctx.prepare_cache_tex(tex, |ctx, unit| {
                // Binding texture
                tex.bind(&self.gl, unit)?;

                ctx.switch_tex += 1;
                Ok(())
            })
        })?;

        self.setup_light(ctx);

        ctx.last_material_bound = Some(Rc::downgrade(&material));

        Ok(())
    }

    #[cfg_attr(feature = "flame_it", flame)]
    fn setup_camera(&self, ctx: &mut EngineContext, modelm: Matrix4<f32>, camera: &Camera) {
        let prog = ctx.prog.upgrade().unwrap();
        // setup_camera
        let perspective = camera.perspective(self.screen_size);

        prog.set("uMVMatrix", camera.v * modelm);
        prog.set("uPMatrix", perspective);

        let skybox_v: Matrix3<_> = Matrix3::from_cols(
            camera.v.x.truncate(),
            camera.v.y.truncate(),
            camera.v.z.truncate(),
        );

        prog.set("uPVMatrix", perspective * camera.v);
        prog.set("uPVSkyboxMatrix", perspective * Matrix4::from(skybox_v));

        prog.set("uNMatrix", modelm.inverse_transform().unwrap().transpose());
        prog.set("uMMatrix", modelm);
        prog.set("uViewPos", camera.eye());
    }

    #[cfg_attr(feature = "flame_it", flame)]
    fn setup_light(&self, ctx: &mut EngineContext) {
        // Setup light
        let prog = ctx.prog.upgrade().unwrap();

        if let Some(ref last_prog) = ctx.last_light_bound {
            if let Some(last_prog) = last_prog.upgrade() {
                if Rc::ptr_eq(&prog, &last_prog) {
                    return;
                }
            }
        }

        ctx.last_light_bound = Some(ctx.prog.clone());

        let light_com = ctx.main_light.as_ref().unwrap();
        let light = light_com.try_as::<Light>().unwrap();

        light.borrow().bind("uDirectionalLight", &prog);
        // So shader needs to have a vs stage light
        light.borrow().bind("uDirectionalLightVS", &prog);

        for (i, plight_com) in ctx.point_lights.iter().enumerate() {
            let plight = plight_com.try_as::<Light>().unwrap();
            let name = format!("uPointLights[{}]", i);
            plight.borrow().bind(&name, &prog);

            let name = format!("uPointLightsVS[{}]", i);
            plight.borrow().bind(&name, &prog);
        }
    }

    #[cfg_attr(feature = "flame_it", flame)]
    fn render_commands(
        &self,
        ctx: &mut EngineContext,
        q: &RenderQueueState,
        camera: &Camera,
        material: Option<&Rc<Material>>,
    ) {
        let gl = &self.gl;

        for cmd in q.commands.iter() {
            let mat = match material.as_ref() {
                Some(&m) => &m,
                None => &cmd.surface.material,
            };

            ctx.states.apply_defaults();
            ctx.states.apply(&q.states);
            ctx.states.apply(&mat.states);
            ctx.states.commit(gl);

            if let Err(err) = self.setup_material(ctx, mat) {
                if let AssetError::NotReady = err {
                    continue;
                }

                panic!(format!("Failed to load material, reason {:?}", err));
            }

            let prog = ctx.prog.upgrade().unwrap();

            let r = ctx.prepare_cache(&cmd.surface.buffer, |ctx| {
                cmd.surface.buffer.bind(&self.gl, &prog)?;
                ctx.switch_mesh += 1;
                Ok(())
            });

            match r {
                Ok(_) => {
                    self.setup_camera(ctx, cmd.model_m, camera);
                    prog.commit(gl);
                    // if let RenderQueue::UI = mat.render_queue
                    {
                        cmd.surface.buffer.render(gl);
                    }

                    cmd.surface.buffer.unbind(gl);
                }
                Err(ref err) => match *err {
                    AssetError::NotReady => (),
                    _ => panic!(format!("Failed to load mesh, reason {:?}", err)),
                },
            }
        }
    }

    fn map_component<T, F>(&self, mut func: F)
    where
        T: 'static + ComponentBased,
        F: FnMut(Rc<RefCell<GameObject>>, Arc<Component>) -> bool,
    {
        for obj in self.objects.iter() {
            let result = obj.upgrade().and_then(|obj| {
                obj.try_borrow()
                    .ok()
                    .and_then(|o| o.find_component::<T>().map(|(_, c)| c.clone()))
            });

            if let Some(com) = result {
                if !func(obj.upgrade().unwrap(), com) {
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
        self.map_component::<T, _>(|_, c| {
            result.push(c);
            true
        });

        result
    }

    pub fn find_component<T>(&self) -> Option<Arc<Component>>
    where
        T: 'static + ComponentBased,
    {
        let mut r = None;
        self.map_component::<T, _>(|_, c| {
            r = Some(c);
            false
        });

        r
    }

    pub fn find_main_light(&self) -> Option<Arc<Component>> {
        self.find_all_components::<Light>()
            .into_iter()
            .filter(|c| {
                let light_com = c.try_as::<Light>().unwrap();
                match *light_com.borrow() {
                    Light::Directional(_) => true,
                    _ => false,
                }
            })
            .nth(0)
    }

    fn prepare_ctx(&self, ctx: &mut EngineContext) {
        // Update all components which need to update
        // Update lights
        self.map_component::<Light, _>(|obj, c| {
            let modelm = obj.borrow().transform.as_global_matrix();

            c.try_as::<Light>().unwrap().borrow_mut().update(&modelm);
            true
        });

        // prepare main light.
        let main_light = self.find_main_light()
            .unwrap_or({ Component::new(Light::new(DirectionalLight::default()), &self.arena) });

        ctx.main_light = Some(main_light);

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
                .map(
                    |c| c.clone()
                )
                .collect();
    }

    fn gather_render_commands(
        &self,
        object: &GameObject,
        cam_pos: &Vector3<f32>,
        update_bounds_only: bool,
        frustum_opt: &Option<Frustum>,
        render_q: &mut RenderQueueList,
        included_render_queues: &Option<BTreeSet<RenderQueue>>,
        eng_stats: &mut Option<&mut EngineStats>,
    ) {
        if !object.active {
            return;
        }

        let result = object.find_component::<Mesh>();
        if let Some((mesh, _)) = result {
            let m = compute_model_m(&*object);
            use math::*;

            // TODO: local scale only ?? should be using global scale??
            let scale = get_max_scale(&object.transform.local_scale());

            for surface in mesh.surfaces.iter() {
                if let &Some(ref included) = included_render_queues {
                    if included.get(&surface.material.render_queue).is_none() {
                        continue;
                    }
                }

                if let &mut Some(ref mut stats) = eng_stats {
                    match surface.material.render_queue {
                        RenderQueue::Transparent => stats.total_transparent_count += 1,
                        RenderQueue::Opaque => stats.total_opaque_count += 1,
                        _ => (),
                    }
                }

                // TODO: should use a material flag to skip
                if let &Some(ref frustum) = frustum_opt {
                    match surface.material.render_queue {
                        RenderQueue::Skybox | RenderQueue::UI => (),
                        _ => {
                            let bounds = surface.buffer.bounds();
                            if bounds.is_none() {
                                continue;
                            }

                            let bounds = bounds.unwrap();
                            let (center, r) = bounds.local_aabb().sphere();

                            let scaled_r = r * scale;
                            let p = m.transform_point(Point3::from_vec(center));

                            if !frustum.collide_sphere(&p.to_vec(), scaled_r) {
                                continue;
                            }

                            if render_q.aabb.is_none() {
                                render_q.aabb = Some(Aabb::empty());
                            }

                            render_q
                                .aabb
                                .as_mut()
                                .unwrap()
                                .merge_sphere(&p.to_vec(), scaled_r);
                        }
                    }
                } else {
                    let bounds = surface.buffer.bounds();
                    if let Some(bounds) = bounds {
                        let (center, r) = bounds.local_aabb().sphere();
                        let p = m.transform_point(Point3::from_vec(center));

                        if render_q.aabb.is_none() {
                            render_q.aabb = Some(Aabb::empty());
                        }

                        render_q
                            .aabb
                            .as_mut()
                            .unwrap()
                            .merge_sphere(&p.to_vec(), r * scale);
                    }
                }

                if !update_bounds_only {
                    let q = render_q
                        .queues
                        .get_mut(&surface.material.render_queue)
                        .unwrap();

                    let cam_dist = (cam_pos - object.transform.global().disp).magnitude();

                    q.commands.push(RenderCommand {
                        surface: surface.clone(),
                        model_m: m,
                        cam_distance: cam_dist,
                    })
                }
            }
        }
    }

    pub fn get_bounds(&self, camera: &Camera) -> Option<Aabb> {
        let render_q = self.gather_all_render_commands(camera, true, None);

        return render_q.aabb;
    }

    fn gather_all_render_commands(
        &self,
        camera: &Camera,
        update_bounds_only: bool,
        mut eng_stats: Option<&mut EngineStats>,
    ) -> RenderQueueList {
        let mut render_q = RenderQueueList::new();
        let objects = &self.objects;

        let frustum = if camera.enable_frustum_culling {
            Some(camera.calc_frustum(self.screen_size))
        } else {
            None
        };

        for obj in objects.iter() {
            obj.upgrade().map(|obj| {
                if let Ok(object) = obj.try_borrow() {
                    self.gather_render_commands(
                        &object,
                        &camera.eye(),
                        update_bounds_only,
                        &frustum,
                        &mut render_q,
                        &camera.included_render_queues,
                        &mut eng_stats,
                    )
                }
            });
        }

        render_q
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn render_pass_with_material(
        &mut self,
        camera: &Camera,
        material: Option<&Rc<Material>>,
        clear_option: ClearOption,
    ) -> EngineStats {
        let mut ctx: EngineContext = EngineContext::new();

        if let Some(ref rt) = camera.render_texture {
            rt.bind_frame_buffer(&self.gl);
        }

        match camera.rect {
            Some(((x, y), (w, h))) => {
                self.gl.viewport(x, y, w, h);
            }
            None => {
                self.gl
                    .viewport(0, 0, self.screen_size.0, self.screen_size.1);
            }
        }

        self.clear(clear_option);

        self.prepare_ctx(&mut ctx);

        // gather commands
        let mut render_q = self.gather_all_render_commands(&camera, false, Some(&mut ctx.stats));

        // Sort the opaque queue
        render_q
            .queues
            .get_mut(&RenderQueue::Opaque)
            .unwrap()
            .sort_by_cam_distance_reverse()
            .sort_by_material();

        // Sort the transparent queue
        render_q
            .queues
            .get_mut(&RenderQueue::Transparent)
            .unwrap()
            .sort_by_cam_distance();

        ctx.stats.surfaces_count = render_q.surface_count() as u32;
        ctx.stats.transparent_count = render_q
            .queues
            .get(&RenderQueue::Transparent)
            .unwrap()
            .commands
            .len() as u32;
        ctx.stats.opaque_count = render_q
            .queues
            .get(&RenderQueue::Opaque)
            .unwrap()
            .commands
            .len() as u32;

        for (_, q) in render_q.queues.iter() {
            self.render_commands(&mut ctx, &q, camera, material);
        }

        if let Some(ref rt) = camera.render_texture {
            rt.unbind_frame_buffer(&self.gl);
        }

        ctx.stats
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn render_pass(&mut self, camera: &Camera, clear_option: ClearOption) -> EngineStats {
        self.render_pass_with_material(camera, None, clear_option)
    }

    pub fn main_camera(&self) -> Option<Arc<Component>> {
        let mut found = self.current_camera.borrow_mut();
        match *found {
            None => *found = self.find_component::<Camera>().map(|c| c.clone()),
            _ => (),
        }

        if let Some(ref c) = *found {
            return Some(c.clone());
        }

        None
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn render(&mut self, clear_option: ClearOption) {
        imgui::pre_render(self);

        if let Some(ref camera) = self.main_camera() {
            self.stats =
                self.render_pass(&camera.try_as::<Camera>().unwrap().borrow(), clear_option);
        } else {
            // We dont have a main camera here, just clean the screen.
            self.clear(clear_option);
        }
    }

    pub fn new(webgl_ctx: WebGLContext, size: (u32, u32), hidpi: f32) -> Engine<A> {
        let gl = WebGLRenderingContext::new(webgl_ctx);

        /*=========Drawing the triangle===========*/

        // Clear the canvas
        gl.clear_color(0.5, 0.5, 0.5, 1.0);

        // Enable alpha blending
        gl.enable(Flag::Blend as i32);

        // Clear the color buffer bit
        gl.clear(BufferBit::Color);
        gl.clear(BufferBit::Depth);
        gl.blend_func(BlendMode::SrcAlpha, BlendMode::OneMinusSrcAlpha);

        // Explict set the clear depth value
        gl.clear_depth(1.0);

        // Set the view port
        gl.viewport(0, 0, size.0, size.1);

        let gui_tree = SceneTree::new();

        Engine {
            gl: gl,
            objects: vec![],
            program_cache: RefCell::new(HashMap::new()),
            asset_system: Box::new(A::new()),
            gui_context: Rc::new(RefCell::new(imgui::Context::new(gui_tree))),
            screen_size: size,
            hidpi: hidpi,
            current_camera: RefCell::new(None),
            stats: Default::default(),
            arena: Rc::new(ComponentArena::new()),
        }
    }

    pub fn begin(&mut self) {
        imgui::begin();

        self.asset_system_mut().step();
    }

    pub fn end(&mut self) {
        // drop all gameobjects if there are no other references
        self.objects.retain(|obj| obj.upgrade().is_some());

        // drop camera cache if it is only by holded by ourself
        let mut cam_mut = self.current_camera.borrow_mut();
        if let Some(ref c) = *cam_mut {
            if Arc::strong_count(&c) == 1 {
                cam_mut.take();
            }
        }
    }

    pub fn capture_frame_buffer(&self) -> Option<image::RgbaImage> {
        use image::imageops;
        use webgl;

        let (width, height) = self.screen_size();

        let mut values: Vec<u8> = vec![0; (width * height * 4) as usize];
        self.gl.read_pixels(
            0,
            0,
            width,
            height,
            webgl::PixelFormat::Rgba,
            webgl::PixelType::UnsignedByte,
            &mut values,
        );

        let img = image::RgbaImage::from_raw(width, height, values);
        // because opengl read_pixels (0,0) is in left bottom,
        // we flip it vertically
        img.map(|img| imageops::flip_vertical(&img))
    }
}

impl<A: AssetSystem> IEngine for Engine<A> {
    fn new_game_object(&mut self, parent: &GameObject) -> Rc<RefCell<GameObject>> {
        let go = parent.tree().new_node(parent, &self.arena);

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

    fn screen_size(&self) -> (u32, u32) {
        self.screen_size
    }

    fn hidpi_factor(&self) -> f32 {
        self.hidpi
    }
}
