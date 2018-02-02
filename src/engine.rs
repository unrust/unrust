use na::*;

use std::mem::size_of;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use uni_app::*;
use webgl::*;

pub struct Camera {
    v: Matrix4<f32>,
    p: Matrix4<f32>,
}

impl Camera {
    pub fn lookat(&mut self, eye: &Point3<f32>, target: &Point3<f32>, up: &Vector3<f32>) {
        self.v = Matrix4::look_at_rh(eye, target, up);
        self.p = Matrix4::new_perspective(800.0 / 600.0, 3.1415 / 4.0, 1.0, 1000.0);
    }

    pub fn new() -> Camera {
        Camera {
            v: Matrix4::identity(),
            p: Matrix4::identity(),
        }
    }
}

pub trait IntoBytes {
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

fn bind_buffer(
    vertices: &Vec<f32>,
    indices: &Vec<u16>,
    normals: &Vec<f32>,
    engine: &Engine,
) -> (WebGLBuffer, WebGLBuffer, WebGLBuffer) {
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

    (vertex_buffer, index_buffer, normal_buffer)
}

pub trait Mesh {
    fn bind(&self, program: &ShaderProgram, gl: &WebGLRenderingContext) {
        let (vertex_buffer, index_buffer, normal_buffer) = self.buffers();

        /*======= Associating shaders to buffer objects =======*/

        // Bind vertex buffer object
        gl.bind_buffer(BufferKind::Array, vertex_buffer);

        // Point an position attribute to the currently bound VBO
        gl.vertex_attrib_pointer(
            program.get_coord(gl, "aVertexPosition"),
            AttributeSize::Three,
            DataType::Float,
            false,
            0,
            0,
        );

        gl.bind_buffer(BufferKind::Array, normal_buffer);
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
        gl.bind_buffer(BufferKind::ElementArray, &index_buffer);
    }

    fn render(&self, gl: &WebGLRenderingContext) {
        gl.draw_elements(
            Primitives::Triangles,
            self.indices().len(),
            DataType::U16,
            0,
        );
    }

    fn vertices(&self) -> &Vec<f32> {
        &self.mesh_buffer().vertices
    }

    fn indices(&self) -> &Vec<u16> {
        &self.mesh_buffer().indices
    }

    fn buffers(&self) -> (&WebGLBuffer, &WebGLBuffer, &WebGLBuffer) {
        let mb = self.mesh_buffer();
        (&mb.vb, &mb.ib, &mb.nb)
    }

    fn mesh_buffer(&self) -> &MeshBuffer;
}

pub struct MeshBuffer {
    #[allow(dead_code)]
    vertices: Vec<f32>,
    indices: Vec<u16>,
    vb: WebGLBuffer,
    ib: WebGLBuffer,
    nb: WebGLBuffer,
}

pub struct CubeMesh(MeshBuffer);

impl CubeMesh {
    pub fn new(engine: &Engine) -> CubeMesh {
        let vertices: Vec<f32> = vec![
            -1.0, -1.0,  1.0,
             1.0, -1.0,  1.0,
             1.0,  1.0,  1.0,
            -1.0,  1.0,  1.0,
            // Back face
            -1.0, -1.0, -1.0,
            -1.0,  1.0, -1.0,
             1.0,  1.0, -1.0,
             1.0, -1.0, -1.0,
            // Top face
            -1.0,  1.0, -1.0,
            -1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0, -1.0,
            // Bottom face
            -1.0, -1.0, -1.0,
             1.0, -1.0, -1.0,
             1.0, -1.0,  1.0,
            -1.0, -1.0,  1.0,
            // Right face
             1.0, -1.0, -1.0,
             1.0,  1.0, -1.0,
             1.0,  1.0,  1.0,
             1.0, -1.0,  1.0,
            // Left face
            -1.0, -1.0, -1.0,
            -1.0, -1.0,  1.0,
            -1.0,  1.0,  1.0,
            -1.0,  1.0, -1.0
            ];

        let indices: Vec<u16> = vec![
            0, 1, 2,      0, 2, 3,    // Front face
            4, 5, 6,      4, 6, 7,    // Back face
            8, 9, 10,     8, 10, 11,  // Top face
            12, 13, 14,   12, 14, 15, // Bottom face
            16, 17, 18,   16, 18, 19, // Right face
            20, 21, 22,   20, 22, 23  // Left face
        ];

        let normals = vec![
            // Front face
             0.0,  0.0,  1.0,
             0.0,  0.0,  1.0,
             0.0,  0.0,  1.0,
             0.0,  0.0,  1.0,
            // Back face
             0.0,  0.0, -1.0,
             0.0,  0.0, -1.0,
             0.0,  0.0, -1.0,
             0.0,  0.0, -1.0,
            // Top face
             0.0,  1.0,  0.0,
             0.0,  1.0,  0.0,
             0.0,  1.0,  0.0,
             0.0,  1.0,  0.0,
            // Bottom face
             0.0, -1.0,  0.0,
             0.0, -1.0,  0.0,
             0.0, -1.0,  0.0,
             0.0, -1.0,  0.0,
            // Right face
             1.0,  0.0,  0.0,
             1.0,  0.0,  0.0,
             1.0,  0.0,  0.0,
             1.0,  0.0,  0.0,
            // Left face
            -1.0,  0.0,  0.0,
            -1.0,  0.0,  0.0,
            -1.0,  0.0,  0.0,
            -1.0,  0.0,  0.0
        ];

        let (vb, ib, nb) = bind_buffer(&vertices, &indices, &normals, engine);

        CubeMesh(MeshBuffer {
            vertices: vertices,
            indices: indices,
            vb,
            ib,
            nb,
        })
    }
}

impl Mesh for CubeMesh {
    fn mesh_buffer(&self) -> &MeshBuffer {
        &self.0
    }
}

pub struct PlaneMesh(MeshBuffer);

impl PlaneMesh {
    pub fn new(engine: &Engine) -> PlaneMesh {
        let vertices: Vec<f32> = vec![
            // Top face
            -10.0,  0.0, -10.0,
            -10.0,  0.0,  10.0,
             10.0,  0.0,  10.0,
             10.0,  0.0, -10.0,
            ];

        let indices: Vec<u16> = vec![
            0, 1, 2, 0, 2, 3 // Top face
        ];

        let normals = vec![
            // Top face
             0.0,  1.0,  0.0,
             0.0,  1.0,  0.0,
             0.0,  1.0,  0.0,
             0.0,  1.0,  0.0,
        ];

        let (vb, ib, nb) = bind_buffer(&vertices, &indices, &normals, engine);

        PlaneMesh(MeshBuffer {
            vertices: vertices,
            indices: indices,
            vb,
            ib,
            nb,
        })
    }
}

impl Mesh for PlaneMesh {
    fn mesh_buffer(&self) -> &MeshBuffer {
        &self.0
    }
}

pub struct GameObject {
    pub transform: Isometry3<f32>,
    pub mesh: Rc<Mesh>,
    pub shader_program: &'static str,
}

pub struct ShaderProgram {
    prog: WebGLProgram,

    coord_map: RefCell<HashMap<&'static str, u32>>,
    uniform_map: RefCell<HashMap<&'static str, Rc<WebGLUniformLocation>>>,
}

impl ShaderProgram {
    pub fn get_coord(&self, gl: &WebGLRenderingContext, s: &'static str) -> u32 {
        let mut m = self.coord_map.borrow_mut();

        match m.get(s) {
            Some(coord) => *coord,
            None => {
                let coord = gl.get_attrib_location(&self.prog, s.into()).unwrap();
                m.insert(s.into(), coord);
                coord
            }
        }
    }

    pub fn get_uniform(
        &self,
        gl: &WebGLRenderingContext,
        s: &'static str,
    ) -> Rc<WebGLUniformLocation> {
        let mut m = self.uniform_map.borrow_mut();

        match m.get(s) {
            Some(u) => u.clone(),
            None => {
                let u = Rc::new(gl.get_uniform_location(&self.prog, s.into()).unwrap());
                {
                    m.insert(s.into(), u.clone());
                }
                u
            }
        }
    }
}

pub struct Engine {
    pub gl: WebGLRenderingContext,
    pub main_camera: Option<Camera>,

    pub objects: Vec<Rc<RefCell<GameObject>>>,

    pub program_cache: RefCell<HashMap<&'static str, Rc<ShaderProgram>>>,
}

struct EngineContext {
    mesh: Option<Rc<Mesh>>,
    switch_mesh: u32,
}

impl Engine {
    pub fn clear(&self) {
        self.gl.clear(BufferBit::Color);
        self.gl.clear(BufferBit::Depth);
        self.gl.clear_color(0.2, 0.2, 0.2, 1.0);
    }

    fn render_object(
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

        if ctx.mesh.is_none() || !Rc::ptr_eq(ctx.mesh.as_ref().unwrap(), &object.mesh) {
            object.mesh.bind(&p, gl);
            ctx.switch_mesh += 1;
        }

        object.mesh.render(gl);
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

                if prog_p.is_none() || object.shader_program != last_prog {
                    // Use the combined shader program object
                    prog_p = Some(self.get_shader_program(&object.shader_program));

                    gl.use_program(&prog_p.as_ref().unwrap().prog);
                    last_prog = object.shader_program;

                    if c > 0 {
                        gl.print("switch shader");
                    }
                    c += 1;
                }

                Engine::render_object(gl, &mut ctx, prog_p.as_ref().unwrap(), &object, camera);

                ctx.mesh = Some(Rc::clone(&object.mesh));
            }
        }
    }

    pub fn add(&mut self, go: Rc<RefCell<GameObject>>) {
        self.objects.push(go)
    }

    fn new_program(&self) -> ShaderProgram {
        let gl = &self.gl;

        /*================ Shaders ====================*/

        // Vertex shader source code
        let vert_code = "       
            attribute vec3 aVertexPosition;
            attribute vec3 aVertexNormal;

            uniform mat4 uMVMatrix;
            uniform mat4 uPMatrix;
            uniform mat4 uNMatrix;
            varying vec3 vColor;
            
            void main(void) {
                gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 1.0);

                vec3 uLightingDirection = vec3(0.4, -0.4, 0);
            
                vec4 transformedNormal = uNMatrix * vec4(aVertexNormal, 1.0);
                float directionalLightWeighting = max(dot(transformedNormal.xyz, -uLightingDirection), 0.0);
            
                vColor = vec3(1.0) * directionalLightWeighting;
            }    
        ";

        // Create a vertex shader object
        let vert_shader = gl.create_shader(ShaderKind::Vertex);

        // Attach vertex shader source code
        gl.shader_source(&vert_shader, vert_code);

        // Compile the vertex shader
        gl.compile_shader(&vert_shader);

        //fragment shader source code
        let frag_code = "
        precision mediump float;

        varying vec3 vColor;
        void main(void) {
            gl_FragColor = vec4(vColor, 1.0);
        }
        ";

        // Create fragment shader object
        let frag_shader = gl.create_shader(ShaderKind::Fragment);

        // Attach fragment shader source code
        gl.shader_source(&frag_shader, frag_code);

        // Compile the fragmentt shader
        gl.compile_shader(&frag_shader);

        // Create a shader program object to store
        // the combined shader program
        let shader_program = gl.create_program();

        // Attach a vertex shader
        gl.attach_shader(&shader_program, &vert_shader);

        // Attach a fragment shader
        gl.attach_shader(&shader_program, &frag_shader);

        // Link both the programs
        gl.link_program(&shader_program);

        let prog = ShaderProgram {
            prog: shader_program,
            coord_map: RefCell::new(HashMap::new()),
            uniform_map: RefCell::new(HashMap::new()),
        };

        let pcoord = prog.get_coord(gl, "aVertexPosition");
        gl.enable_vertex_attrib_array(pcoord);

        let ncoord = prog.get_coord(gl, "aVertexNormal");
        gl.enable_vertex_attrib_array(ncoord);

        prog
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
