mod mesh;
mod primitives;
mod shader_program;
mod camera;
mod material;
mod game_object;
mod texture;

use na::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use uni_app::*;
use webgl::*;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

pub use self::mesh::{Mesh, MeshBuffer};
pub use self::primitives::PrimitiveMesh;
pub use self::shader_program::ShaderProgram;
pub use self::camera::Camera;
pub use self::material::Material;
pub use self::texture::Texture;
pub use self::game_object::{Component, ComponentBased, GameObject};

pub struct Engine {
    pub gl: WebGLRenderingContext,
    pub main_camera: Option<Camera>,

    pub objects: Vec<Rc<RefCell<GameObject>>>,

    pub program_cache: RefCell<HashMap<&'static str, Rc<ShaderProgram>>>,
}

#[derive(Default)]
struct EngineContext<'a> {
    mesh: Option<u64>,
    switch_mesh: u32,
    switch_prog: u32,

    last_prog: &'a str,
    current_prog: Option<Rc<ShaderProgram>>,
}

impl Engine {
    pub fn clear(&self) {
        self.gl.clear(BufferBit::Color);
        self.gl.clear(BufferBit::Depth);
        self.gl.clear_color(0.2, 0.2, 0.2, 1.0);
    }

    fn setup_material(&self, ctx: &mut EngineContext, material: &Material) {
        if ctx.current_prog.is_none() || material.program != ctx.last_prog {
            // Use the combined shader program object
            ctx.current_prog = Some(self.get_shader_program(&material.program));
            ctx.current_prog.as_ref().map(|p| p.use_program(&self.gl));
            ctx.last_prog = material.program;
            ctx.switch_prog += 1;
        }

        let curr = &mut ctx.current_prog;
        // Binding texture
        material.texture.bind(self, curr.as_ref().unwrap());
    }

    fn render_object(
        &self,
        gl: &WebGLRenderingContext,
        ctx: &mut EngineContext,
        object: &GameObject,
        camera: &Camera,
    ) {
        // Setup Matrices
        let modelm = object.transform.to_homogeneous();

        let p = ctx.current_prog.as_ref().unwrap();

        let umv = p.get_uniform(gl, "uMVMatrix");
        gl.uniform_matrix_4fv(&umv, &(camera.v * modelm).into());

        let up = p.get_uniform(gl, "uPMatrix");
        gl.uniform_matrix_4fv(&up, &camera.p.into());

        let normal_mat = (modelm).try_inverse().unwrap().transpose();

        let nm = p.get_uniform(gl, "uNMatrix");
        gl.uniform_matrix_4fv(&nm, &normal_mat.into());

        // Setup Mesh
        let (mesh, com) = object.get_component_by_type::<Mesh>().unwrap();

        if ctx.mesh.is_none() || ctx.mesh.unwrap() != com.id() {
            mesh.bind(self, &p);
            ctx.switch_mesh += 1;
        }

        mesh.render(gl);
    }

    pub fn get_shader_program(&self, name: &'static str) -> Rc<ShaderProgram> {
        let mut cache = self.program_cache.borrow_mut();

        match cache.get_mut(name) {
            Some(prog) => prog.clone(),
            None => {
                let u = Rc::new(self.new_program());
                cache.insert(name, u.clone());
                u
            }
        }
    }

    pub fn render(&mut self) {
        self.clear();
        let objects = &self.objects;
        let gl = &self.gl;

        if let &Some(camera) = &self.main_camera.as_ref() {
            let mut ctx: EngineContext = Default::default();

            for obj in objects.iter() {
                let object = obj.borrow();
                let (material, _) = object.get_component_by_type::<Material>().unwrap();

                {
                    self.setup_material(&mut ctx, material);
                }

                self.render_object(gl, &mut ctx, &object, camera);

                let (_, meshcom) = object.get_component_by_type::<Mesh>().unwrap();
                ctx.mesh = Some(meshcom.id());
            }
        }
    }

    pub fn new_gameobject(&mut self, transform: &Isometry3<f32>) -> Rc<RefCell<GameObject>> {
        let go = Rc::new(RefCell::new(GameObject {
            transform: *transform,
            components: vec![],
        }));

        self.objects.push(go.clone());
        go
    }

    pub fn next_component_id() -> u64 {
        static CURR_COMPONENT_COUNTER: AtomicU32 = AtomicU32::new(1);;

        CURR_COMPONENT_COUNTER.fetch_add(1, Ordering::SeqCst) as u64
    }

    fn new_program(&self) -> ShaderProgram {
        ShaderProgram::new(&self.gl)
    }

    pub fn new(app: &App, size: (u32, u32)) -> Engine {
        let gl = WebGLRenderingContext::new(app.canvas());

        /*=========Drawing the triangle===========*/

        // Clear the canvas
        gl.clear_color(0.5, 0.5, 0.5, 0.9);

        // Enable the depth test
        gl.enable(Flag::DepthTest);

        // Clear the color buffer bit
        gl.clear(BufferBit::Color);
        gl.clear(BufferBit::Depth);

        // Set the view port
        gl.viewport(0, 0, size.0, size.1);

        Engine {
            gl: gl,
            main_camera: None,
            objects: vec![],
            program_cache: RefCell::new(HashMap::new()),
        }
    }
}
