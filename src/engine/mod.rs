mod mesh;
mod primitives;
mod shader_program;
mod camera;
mod material;

use na::*;
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use std::collections::HashMap;
use uni_app::*;
use webgl::*;
use std::any::{Any, TypeId};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

pub use self::mesh::{Mesh, MeshBuffer};
pub use self::primitives::PrimitiveMesh;
pub use self::shader_program::ShaderProgram;
pub use self::camera::Camera;
pub use self::material::Material;

pub trait Component: Any {
    fn id(&self) -> u64;
    fn typeid(&self) -> TypeId;

    fn as_any(&self) -> &Any;
}

pub struct ComponentType<T>
where
    T: ComponentBased,
{
    com: Rc<T>,
    id: u64,
}

impl<T> Component for ComponentType<T>
where
    T: 'static + ComponentBased,
{
    fn id(&self) -> u64 {
        self.id
    }

    fn typeid(&self) -> TypeId {
        return TypeId::of::<T>();
    }

    fn as_any(&self) -> &Any {
        self
    }
}

pub trait ComponentBased {}

impl Component {
    fn try_into<T>(&self) -> Option<&T>
    where
        T: 'static + ComponentBased,
    {
        let a = self.as_any();
        match a.downcast_ref::<ComponentType<T>>() {
            Some(t) => Some(t.com.as_ref()),
            _ => None,
        }
    }

    fn new<T>(value: T) -> Arc<Component>
    where
        T: 'static + ComponentBased,
    {
        let c = ComponentType {
            com: Rc::new(value),
            id: Engine::next_com_id(),
        };

        Arc::new(c)
    }
}

pub struct GameObject {
    pub transform: Isometry3<f32>,
    pub components: Vec<Arc<Component>>,
}

impl GameObject {
    pub fn get_component_by_type<T>(&self) -> Option<(&T, &Component)>
    where
        T: 'static + ComponentBased,
    {
        let typeid = TypeId::of::<T>();

        match self.components.iter().find(|c| c.typeid() == typeid) {
            Some(c) => {
                let com: &Component = c.as_ref();
                Some((com.try_into::<T>().unwrap(), com))
            }
            _ => None,
        }
    }

    pub fn add_component(&mut self, c: Arc<Component>) {
        self.components.push(c.clone());
    }
}

pub struct Engine {
    pub gl: WebGLRenderingContext,
    pub main_camera: Option<Camera>,

    pub objects: Vec<Rc<RefCell<GameObject>>>,

    pub program_cache: RefCell<HashMap<&'static str, Rc<ShaderProgram>>>,
}

struct EngineContext {
    mesh: Option<u64>,
    switch_mesh: u32,
}

impl Engine {
    pub fn clear(&self) {
        self.gl.clear(BufferBit::Color);
        self.gl.clear(BufferBit::Depth);
        self.gl.clear_color(0.2, 0.2, 0.2, 1.0);
    }

    fn render_object(
        &self,
        gl: &WebGLRenderingContext,
        ctx: &mut EngineContext,
        p: &ShaderProgram,
        object: &GameObject,
        camera: &Camera,
    ) {
        let modelm = object.transform.to_homogeneous();

        let umv = p.get_uniform(gl, "uMVMatrix");
        gl.uniform_matrix_4fv(&umv, &(camera.v * modelm).into());

        let up = p.get_uniform(gl, "uPMatrix");
        gl.uniform_matrix_4fv(&up, &camera.p.into());

        let normal_mat = (camera.v * modelm).try_inverse().unwrap().transpose();

        let nm = p.get_uniform(gl, "uNMatrix");
        gl.uniform_matrix_4fv(&nm, &normal_mat.into());

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
                {
                    cache.insert(name, u.clone());
                }
                u
            }
        }
    }

    pub fn render(&mut self) {
        self.clear();
        let objects = &self.objects;
        let gl = &self.gl;

        let mut last_prog = "";
        let mut prog_p = None;

        if let &Some(camera) = &self.main_camera.as_ref() {
            let mut ctx = EngineContext {
                mesh: None,
                switch_mesh: 0,
            };
            let mut c = 0;

            for obj in objects.iter() {
                let object = obj.borrow();
                let (material, _) = object.get_component_by_type::<Material>().unwrap();

                if prog_p.is_none() || material.program != last_prog {
                    // Use the combined shader program object
                    prog_p = Some(self.get_shader_program(&material.program));
                    prog_p.as_ref().map(|p| p.use_program(gl));

                    last_prog = material.program;

                    if c > 0 {
                        gl.print("switch shader");
                    }
                    c += 1;
                }

                self.render_object(gl, &mut ctx, prog_p.as_ref().unwrap(), &object, camera);

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

    pub fn next_com_id() -> u64 {
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
