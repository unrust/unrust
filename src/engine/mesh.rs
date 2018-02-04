use webgl::*;
use std::mem::size_of;

use Engine;
use ShaderProgram;
use ComponentBased;
use std::cell::RefCell;

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

pub struct Mesh {
    pub mesh_buffer: MeshBuffer,
    pub gl_state: RefCell<Option<MeshGLState>>,
}

impl ComponentBased for Mesh {}

pub struct MeshGLState {
    pub vb: WebGLBuffer,
    pub ib: WebGLBuffer,
    pub nb: WebGLBuffer,
}

impl Mesh {
    pub fn bind(&self, engine: &Engine, program: &ShaderProgram) {
        let gl = &engine.gl;

        self.prepare(engine);

        let state_option = self.gl_state.borrow();
        let state = state_option.as_ref().unwrap();

        /*======= Associating shaders to buffer objects =======*/

        // Bind vertex buffer object
        gl.bind_buffer(BufferKind::Array, &state.vb);

        // Point an position attribute to the currently bound VBO
        gl.vertex_attrib_pointer(
            program.get_coord(gl, "aVertexPosition"),
            AttributeSize::Three,
            DataType::Float,
            false,
            0,
            0,
        );

        gl.bind_buffer(BufferKind::Array, &state.nb);
        // Point an position attribute to the currently bound VBO
        gl.vertex_attrib_pointer(
            program.get_coord(gl, "aVertexNormal"),
            AttributeSize::Three,
            DataType::Float,
            false,
            0,
            0,
        );

        // Bind index buffer object
        gl.bind_buffer(BufferKind::ElementArray, &state.ib);
    }

    pub fn render(&self, gl: &WebGLRenderingContext) {
        gl.draw_elements(
            Primitives::Triangles,
            self.indices().len(),
            DataType::U16,
            0,
        );
    }

    pub fn indices(&self) -> &Vec<u16> {
        &self.mesh_buffer.indices
    }

    pub fn prepare(&self, engine: &Engine) {
        if self.gl_state.borrow().is_none() {
            self.gl_state.replace(Some(mesh_bind_buffer(
                &self.mesh_buffer.vertices,
                &self.mesh_buffer.indices,
                &self.mesh_buffer.normals,
                engine,
            )));
        }
    }
}

pub struct MeshBuffer {
    #[allow(dead_code)]
    pub vertices: Vec<f32>,
    pub indices: Vec<u16>,
    pub normals: Vec<f32>,
}

pub fn mesh_bind_buffer(
    vertices: &Vec<f32>,
    indices: &Vec<u16>,
    normals: &Vec<f32>,
    engine: &Engine,
) -> MeshGLState {
    let gl = &engine.gl;

    // Create an empty buffer object to store vertex buffer
    let vertex_buffer = gl.create_buffer();
    {
        // Bind appropriate array buffer to it
        gl.bind_buffer(BufferKind::Array, &vertex_buffer);

        // Pass the vertex data to the buffer
        let cv = vertices.clone();
        gl.buffer_data(BufferKind::Array, &cv.into_bytes(), DrawMode::Static);

        // Unbind the buffer
        gl.unbind_buffer(BufferKind::Array);
    }

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

    // Create an Normal Buffer
    let normal_buffer = gl.create_buffer();
    {
        // Bind appropriate array buffer to it
        gl.bind_buffer(BufferKind::Array, &normal_buffer);

        let ns = normals.clone();
        gl.buffer_data(BufferKind::Array, &ns.into_bytes(), DrawMode::Static);

        // Unbind the buffer
        gl.unbind_buffer(BufferKind::Array);
    }

    MeshGLState {
        vb: vertex_buffer,
        ib: index_buffer,
        nb: normal_buffer,
    }
}
