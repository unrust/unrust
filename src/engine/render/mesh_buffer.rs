use webgl::*;
use std::mem::size_of;

use super::ShaderProgram;
use engine::asset::{Asset, AssetResult, AssetSystem, FileFuture, LoadableAsset, Resource};
use engine::render::mesh::MeshBound;

use std::cell::RefCell;
use std::rc::Rc;
use std::cell::Cell;
use std::f32::{MAX, MIN};
use na::Vector3;

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

struct MeshGLState {
    pub vao: WebGLVertexArray,
    pub vb: WebGLBuffer,
    pub uvb: Option<WebGLBuffer>,

    pub nb: Option<WebGLBuffer>,
    pub tb: Option<WebGLBuffer>,
    pub btb: Option<WebGLBuffer>,

    pub ib: WebGLBuffer,
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

pub struct MeshBuffer {
    data: Resource<MeshData>,
    gl_state: RefCell<Option<MeshGLState>>,
    bounds: Cell<Option<MeshBound>>,
}

impl Asset for MeshBuffer {
    type Resource = Resource<MeshData>;

    fn new_from_resource(r: Self::Resource) -> Rc<Self> {
        Rc::new(MeshBuffer {
            data: r,
            gl_state: Default::default(),
            bounds: Default::default(),
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
    program: &ShaderProgram,
    name: &str,
    asize: AttributeSize,
) {
    gl.bind_buffer(BufferKind::Array, buffer);

    // Point an position attribute to the currently bound VBO
    if let Some(coord) = program.attrib_loc(gl, name) {
        gl.enable_vertex_attrib_array(coord);
        gl.vertex_attrib_pointer(coord, asize, DataType::Float, false, 0, 0);
    }
}

impl MeshBuffer {
    pub fn prepare(&self, gl: &WebGLRenderingContext) -> AssetResult<()> {
        if self.gl_state.borrow().is_some() {
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
        let mut min = Vector3::new(MAX, MAX, MAX);
        let mut max = Vector3::new(MIN, MIN, MIN);
        let mut r: f32 = 0.0;

        let data = self.data.try_borrow().ok()?;

        for (i, v) in data.vertices.iter().enumerate() {
            min[i % 3] = v.min(min[i % 3]);
            max[i % 3] = v.max(max[i % 3]);

            if i % 3 == 0 {
                let d = Vector3::from_row_slice(&data.vertices[i..i + 3]).norm();
                r = r.max(d);
            }
        }

        Some(MeshBound { min, max, r })
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

    pub fn bind(&self, gl: &WebGLRenderingContext, program: &ShaderProgram) -> AssetResult<()> {
        self.prepare(gl)?;

        let state_option = self.gl_state.borrow();
        let state = state_option.as_ref().unwrap();

        /*======= Associating shaders to buffer objects =======*/
        gl.bind_vertex_array(&state.vao);

        // Bind vertex buffer object
        bind_buffer(
            gl,
            &state.vb,
            program,
            "aVertexPosition",
            AttributeSize::Three,
        );

        if let Some(ref uvb) = state.uvb {
            bind_buffer(gl, uvb, program, "aTextureCoord", AttributeSize::Two);
        }

        if let Some(ref nb) = state.nb {
            bind_buffer(gl, nb, program, "aVertexNormal", AttributeSize::Three);
        }

        if let Some(ref tb) = state.tb {
            bind_buffer(gl, tb, program, "aVertexTangent", AttributeSize::Three);
        }

        if let Some(ref btb) = state.btb {
            bind_buffer(gl, btb, program, "aVertexBitangent", AttributeSize::Three);
        }

        // Bind index buffer object
        gl.bind_buffer(BufferKind::ElementArray, &state.ib);

        Ok(())
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn render(&self, gl: &WebGLRenderingContext) {
        let data = self.data.try_borrow().unwrap();

        gl.draw_elements(Primitives::Triangles, data.indices.len(), DataType::U16, 0);
    }

    pub fn unbind(&self, gl: &WebGLRenderingContext) {
        let state_option = self.gl_state.borrow();
        let state = state_option.as_ref().unwrap();
        gl.unbind_vertex_array(&state.vao);
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
    let uv_buffer = uvs.as_ref().map(|ref data| bind_f32_array(gl, data));
    let normal_buffer = normals.as_ref().map(|ref data| bind_f32_array(gl, data));
    let tangent_buffer = tangents.as_ref().map(|ref data| bind_f32_array(gl, data));
    let bitangent_buffer = bitangents.as_ref().map(|ref data| bind_f32_array(gl, data));

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
    }
}
