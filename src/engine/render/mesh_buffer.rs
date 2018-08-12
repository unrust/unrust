use std::mem::size_of;
use uni_gl::*;

use super::ShaderProgram;
use engine::asset::{Asset, AssetResult, AssetSystem, FileFuture, LoadableAsset, Resource};
use engine::core::Aabb;
use engine::render::mesh::MeshBound;
use engine::render::shader_program::ShaderAttrib;

use math::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::f32::{MAX, MIN};
use std::rc::Rc;
use std::rc::Weak;

trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
}

impl<T> IntoBytes for Vec<T> {
    fn into_bytes(self) -> Vec<u8> {
        let len = size_of::<T>() * self.len();
        unsafe {
            let slice = self.into_boxed_slice();
            Vec::<u8>::from_raw_parts(Box::into_raw(slice) as _, len, len)
        }
    }
}

enum RebindAction {
    Vertices,
    UV,
    Normal,
    Tangent,
    Bitangent,
    Indices,
}

struct MeshGLState {
    pub vao: WebGLVertexArray,
    pub vb: WebGLBuffer,
    pub uvb: Option<WebGLBuffer>,

    pub nb: Option<WebGLBuffer>,
    pub tb: Option<WebGLBuffer>,
    pub btb: Option<WebGLBuffer>,

    pub ib: WebGLBuffer,
    pub gl: WebGLRenderingContext,

    pub rebind_actions: Vec<RebindAction>,
}

impl MeshGLState {
    fn rebind_buffer(
        &mut self,
        data: &MeshData,
        tt: &RebindAction,
    ) -> (BufferKind, Vec<u8>, &mut WebGLBuffer) {
        match *tt {
            RebindAction::Vertices => (
                BufferKind::Array,
                data.vertices.clone().into_bytes(),
                &mut self.vb,
            ),
            RebindAction::UV => (
                BufferKind::Array,
                data.uvs.clone().unwrap().into_bytes(),
                self.uvb.as_mut().unwrap(),
            ),
            RebindAction::Normal => (
                BufferKind::Array,
                data.normals.clone().unwrap().into_bytes(),
                self.nb.as_mut().unwrap(),
            ),
            RebindAction::Tangent => (
                BufferKind::Array,
                data.tangents.clone().unwrap().into_bytes(),
                self.tb.as_mut().unwrap(),
            ),
            RebindAction::Bitangent => (
                BufferKind::Array,
                data.bitangents.clone().unwrap().into_bytes(),
                self.btb.as_mut().unwrap(),
            ),
            RebindAction::Indices => (
                BufferKind::ElementArray,
                data.indices.clone().into_bytes(),
                &mut self.ib,
            ),
        }
    }

    fn rebind(&mut self, actions: &Vec<RebindAction>, data: &MeshData, gl: &WebGLRenderingContext) {
        for action in actions.iter() {
            let (k, p, buf) = self.rebind_buffer(data, action);

            gl.bind_buffer(k, &buf);
            gl.buffer_data(k, &p, DrawMode::Static);
            gl.unbind_buffer(k);
        }
    }
}

impl Drop for MeshGLState {
    fn drop(&mut self) {
        self.gl.delete_buffer(&self.vb);
        self.uvb.as_ref().map(|b| self.gl.delete_buffer(&b));
        self.nb.as_ref().map(|b| self.gl.delete_buffer(&b));
        self.tb.as_ref().map(|b| self.gl.delete_buffer(&b));
        self.btb.as_ref().map(|b| self.gl.delete_buffer(&b));
        self.gl.delete_buffer(&self.ib);

        self.gl.delete_vertex_array(&self.vao);
    }
}

#[derive(Default, Debug)]
pub struct MeshData {
    pub vertices: Vec<f32>,
    pub uvs: Option<Vec<f32>>,
    pub normals: Option<Vec<f32>>,

    pub tangents: Option<Vec<f32>>,
    pub bitangents: Option<Vec<f32>>,

    pub indices: Vec<u16>,
}

impl MeshData {
    pub fn compute_bound(&self) -> MeshBound {
        let mut min = Vector3::new(MAX, MAX, MAX);
        let mut max = Vector3::new(MIN, MIN, MIN);
        let mut r: f32 = 0.0;

        for (i, v) in self.vertices.iter().enumerate() {
            min[i % 3] = v.min(min[i % 3]);
            max[i % 3] = v.max(max[i % 3]);

            if i % 3 == 0 {
                let vs = &self.vertices[i..i + 3];
                let d = vec3(vs[0], vs[1], vs[2]).magnitude();
                r = r.max(d);
            }
        }

        MeshBound {
            aabb: Aabb { min, max },
            r,
        }
    }

    pub fn translate(&mut self, disp: Vector3f) {
        for (i, v) in self.vertices.iter_mut().enumerate() {
            *v += disp[i % 3];
        }
    }
}

pub struct MeshBuffer {
    data: Resource<MeshData>,
    gl_state: RefCell<Option<MeshGLState>>,
    bounds: Cell<Option<MeshBound>>,

    bound_prog: RefCell<Weak<ShaderProgram>>,
}

impl Asset for MeshBuffer {
    type Resource = Resource<MeshData>;

    fn new_from_resource(r: Self::Resource) -> Rc<Self> {
        Rc::new(MeshBuffer {
            data: r,
            gl_state: Default::default(),
            bounds: Default::default(),
            bound_prog: RefCell::new(Weak::new()),
        })
    }
}

impl LoadableAsset for MeshBuffer {
    fn load<T>(asys: &T, mut files: Vec<FileFuture>) -> Self::Resource
    where
        T: AssetSystem + Clone + 'static,
    {
        Self::load_resource::<MeshData, T>(asys.clone(), files.remove(0))
    }

    fn gather<T: AssetSystem>(asys: &T, fname: &str) -> Vec<FileFuture> {
        vec![asys.new_file(fname)]
    }
}

pub fn bind_buffer(
    gl: &WebGLRenderingContext,
    buffer: &WebGLBuffer,
    coord: u32,
    asize: AttributeSize,
) {
    gl.bind_buffer(BufferKind::Array, buffer);

    gl.enable_vertex_attrib_array(coord);
    gl.vertex_attrib_pointer(coord, asize, DataType::Float, false, 0, 0);
}

impl MeshBuffer {
    pub fn update_mesh_data(&self, mesh_data: MeshData) {
        let mut actions = Vec::new();

        match self.data.try_borrow() {
            Err(_) => {}
            Ok(_da) => {
                actions.push(RebindAction::Vertices);

                mesh_data.uvs.as_ref().map(|_| {
                    actions.push(RebindAction::UV);
                });

                mesh_data.normals.as_ref().map(|_| {
                    actions.push(RebindAction::Normal);
                });

                mesh_data.tangents.as_ref().map(|_| {
                    actions.push(RebindAction::Tangent);
                });

                mesh_data.bitangents.as_ref().map(|_| {
                    actions.push(RebindAction::Bitangent);
                });

                actions.push(RebindAction::Indices);
            }
        };

        self.data.replace(mesh_data);

        // check whether the state is ready
        match *self.gl_state.borrow_mut() {
            None => {}
            Some(ref mut state) => {
                state.rebind_actions.append(&mut actions);
                *self.bound_prog.borrow_mut() = Weak::new();
            }
        }
    }

    pub fn prepare(&self, gl: &WebGLRenderingContext) -> AssetResult<()> {
        if let Some(ref mut state) = *self.gl_state.borrow_mut() {
            if state.rebind_actions.len() > 0 {
                gl.bind_vertex_array(&state.vao);

                let data = self.data.try_borrow()?;
                let rebind_actions = state.rebind_actions.drain(..).collect();

                // Rebind the mesh
                state.rebind(&rebind_actions, &data, gl);
            }

            return Ok(());
        }

        let data = self.data.try_borrow()?;

        self.gl_state.replace(Some(mesh_bind_buffer(
            &data.vertices,
            &data.uvs,
            &data.normals,
            &data.tangents,
            &data.bitangents,
            &data.indices,
            gl,
        )));

        Ok(())
    }

    fn compute_bounds(&self) -> Option<MeshBound> {
        let data = self.data.try_borrow().ok()?;
        Some(data.compute_bound())
    }

    /// bounds return (vmin, vmax)
    pub fn bounds(&self) -> Option<MeshBound> {
        let bounds = self.bounds.get();

        match bounds {
            r @ Some(_) => r,
            None => {
                self.bounds.set(self.compute_bounds());
                self.bounds.get()
            }
        }
    }

    pub fn bind(&self, gl: &WebGLRenderingContext, program: &Rc<ShaderProgram>) -> AssetResult<()> {
        self.prepare(gl)?;

        let state_option = self.gl_state.borrow();
        let state = state_option.as_ref().unwrap();

        /*======= Associating shaders to buffer objects =======*/
        gl.bind_vertex_array(&state.vao);

        if gl.is_webgl2 {
            if let Some(_) = self.bound_prog.borrow().upgrade() {
                return Ok(());
            }
        }

        // Bind vertex buffer object
        // "aVertexPosition"
        bind_buffer(
            gl,
            &state.vb,
            ShaderAttrib::Position as u32,
            AttributeSize::Three,
        );

        // "aTextureCoord"
        if let Some(ref uvb) = state.uvb {
            bind_buffer(gl, uvb, ShaderAttrib::UV0 as u32, AttributeSize::Two);
        }

        // "aVertexNormal"
        if let Some(ref nb) = state.nb {
            bind_buffer(gl, nb, ShaderAttrib::Normal as u32, AttributeSize::Three);
        }

        // "aVertexTangent"
        if let Some(ref tb) = state.tb {
            bind_buffer(gl, tb, ShaderAttrib::Tangent as u32, AttributeSize::Three);
        }

        // "aVertexBitangent"
        if let Some(ref btb) = state.btb {
            bind_buffer(
                gl,
                btb,
                ShaderAttrib::Bitangent as u32,
                AttributeSize::Three,
            );
        }

        // Bind index buffer object
        gl.bind_buffer(BufferKind::ElementArray, &state.ib);

        *self.bound_prog.borrow_mut() = Rc::downgrade(program);

        Ok(())
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn render(&self, gl: &WebGLRenderingContext) {
        let data = self.data.try_borrow().unwrap();

        gl.draw_elements(Primitives::Triangles, data.indices.len(), DataType::U16, 0);
    }

    pub fn unbind(&self, _gl: &WebGLRenderingContext) {
        //let state_option = self.gl_state.borrow();
        //let state = state_option.as_ref().unwrap();

        // Normally we do not need to unbind a vertex array
        //gl.unbind_vertex_array(&state.vao);
    }
}

fn bind_f32_array(gl: &WebGLRenderingContext, data: &Vec<f32>) -> WebGLBuffer {
    // Create an empty buffer object to store vertex buffer
    let vb = gl.create_buffer();
    {
        // Bind appropriate array buffer to it
        gl.bind_buffer(BufferKind::Array, &vb);

        // Pass the vertex data to the buffer
        let cv = data.clone();
        gl.buffer_data(BufferKind::Array, &cv.into_bytes(), DrawMode::Static);

        // Unbind the buffer
        gl.unbind_buffer(BufferKind::Array);
    }

    vb
}

fn mesh_bind_buffer(
    vertices: &Vec<f32>,
    uvs: &Option<Vec<f32>>,
    normals: &Option<Vec<f32>>,
    tangents: &Option<Vec<f32>>,
    bitangents: &Option<Vec<f32>>,
    indices: &Vec<u16>,
    gl: &WebGLRenderingContext,
) -> MeshGLState {
    // some opengl 3.x core profile require a VAO. See issue #11
    let vao = gl.create_vertex_array();
    gl.bind_vertex_array(&vao);

    let vertex_buffer = bind_f32_array(&gl, vertices);
    let uv_buffer = uvs.as_ref().map(|data| bind_f32_array(gl, data));
    let normal_buffer = normals.as_ref().map(|data| bind_f32_array(gl, data));
    let tangent_buffer = tangents.as_ref().map(|data| bind_f32_array(gl, data));
    let bitangent_buffer = bitangents.as_ref().map(|data| bind_f32_array(gl, data));

    // Create an empty buffer object to store Index buffer
    let index_buffer = gl.create_buffer();
    {
        // Bind appropriate array buffer to it
        gl.bind_buffer(BufferKind::ElementArray, &index_buffer);

        // Pass the vertex data to the buffer
        let ci = indices.clone();
        gl.buffer_data(BufferKind::ElementArray, &ci.into_bytes(), DrawMode::Static);

        // Unbind the buffer
        gl.unbind_buffer(BufferKind::ElementArray);
    }

    MeshGLState {
        vao,
        vb: vertex_buffer,
        uvb: uv_buffer,

        nb: normal_buffer,
        tb: tangent_buffer,
        btb: bitangent_buffer,

        ib: index_buffer,
        gl: gl.clone(),

        rebind_actions: Vec::new(),
    }
}
