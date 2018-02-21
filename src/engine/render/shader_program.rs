use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use webgl::{ShaderKind as WebGLShaderKind, WebGLProgram, WebGLRenderingContext,
            WebGLUniformLocation, IS_GL_ES};

use na::{Matrix4, Vector3};
use engine::asset::{Asset, File};
use std::fmt::Debug;

use engine::render::shader::{Shader, ShaderKind as Kind};

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::mem::size_of;

#[derive(Debug)]
pub struct ShaderProgramGLState {
    prog: WebGLProgram,
}

trait IntoBytes {
    fn into_bytes(&self) -> Vec<u8>;
}

impl<T: Clone> IntoBytes for [T] {
    fn into_bytes(&self) -> Vec<u8> {
        let v = self.to_vec();
        let len = size_of::<T>() * v.len();
        unsafe {
            let slice = v.into_boxed_slice();
            Vec::<u8>::from_raw_parts(Box::into_raw(slice) as _, len, len)
        }
    }
}

#[derive(Debug, Default)]
pub struct ShaderProgram {
    gl_state: RefCell<Option<ShaderProgramGLState>>,

    coord_map: RefCell<HashMap<String, Option<u32>>>,
    uniform_map: RefCell<HashMap<String, Option<Rc<WebGLUniformLocation>>>>,

    vs_shader: Shader,
    fs_shader: Shader,

    pending_uniforms: RefCell<HashMap<String, Box<IntoUniformSetter>>>,
    committed_unforms: RefCell<HashMap<String, u64>>,
}

pub trait IntoUniformSetter: Debug {
    fn set(&self, gl: &WebGLRenderingContext, loc: &WebGLUniformLocation);

    fn to_hash(&self) -> u64;
}

impl IntoUniformSetter for Matrix4<f32> {
    fn set(&self, gl: &WebGLRenderingContext, loc: &WebGLUniformLocation) {
        let m = *self;
        gl.uniform_matrix_4fv(&loc, &m.into());
    }

    fn to_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        s.write(&self.as_slice().into_bytes());
        s.finish()
    }
}

impl IntoUniformSetter for f32 {
    fn set(&self, gl: &WebGLRenderingContext, loc: &WebGLUniformLocation) {
        gl.uniform_1f(&loc, *self);
    }

    fn to_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        let f: f32 = *self;
        s.write(&[f].into_bytes());
        s.finish()
    }
}

impl IntoUniformSetter for i32 {
    fn set(&self, gl: &WebGLRenderingContext, loc: &WebGLUniformLocation) {
        gl.uniform_1i(&loc, *self);
    }

    fn to_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        s.write_i32(*self);
        s.finish()
    }
}

impl IntoUniformSetter for Vector3<f32> {
    fn set(&self, gl: &WebGLRenderingContext, loc: &WebGLUniformLocation) {
        gl.uniform_3f(&loc, (self.x, self.y, self.z));
    }

    fn to_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        s.write(&self.as_slice().into_bytes());
        s.finish()
    }
}

impl ShaderProgram {
    pub fn new_default() -> ShaderProgram {
        Self::new(("phong_vs.glsl", DEFAULT_VS), ("phong_fs.glsl", DEFAULT_FS))
    }

    pub fn new_default_ui() -> ShaderProgram {
        Self::new(("ui_fs.glsl", DEFAULT_UI_VS), ("ui_vs.fs", DEFAULT_UI_FS))
    }

    pub fn new((vs_filename, vs): (&str, &str), (fs_filename, fs): (&str, &str)) -> ShaderProgram {
        let mut program: ShaderProgram = Default::default();

        let mut avs = vs.to_string();
        let mut afs = fs.to_string();

        if !IS_GL_ES {
            avs = "#version 130\n".to_string() + &avs;
            afs = "#version 130\n".to_string() + &afs;
        } else {
            afs = ("precision highp float;\n").to_string() + &afs;
        }

        // Vertex shader source code
        program.vs_shader = Shader::new(Kind::Vertex, vs_filename, &avs);

        //fragment shader source code
        program.fs_shader = Shader::new(Kind::Fragment, fs_filename, &afs);

        program
    }

    pub fn bind(&self, gl: &WebGLRenderingContext) {
        self.prepare(gl);

        let gl_state = self.gl_state.borrow();
        gl.use_program(&gl_state.as_ref().unwrap().prog);

        // after use, we should clean up the committed uniform state.
        // https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glUniform.xhtml
        self.committed_unforms.borrow_mut().clear();
    }

    fn prepare(&self, gl: &WebGLRenderingContext) {
        let is_none = self.gl_state.borrow().is_none();

        if is_none {
            {
                let state = Some(ShaderProgramGLState::new(
                    gl,
                    &self.vs_shader.code,
                    &self.fs_shader.code,
                ));
                *self.gl_state.borrow_mut() = state;
            }
        }
    }

    pub fn attrib_loc(&self, gl: &WebGLRenderingContext, s: &str) -> Option<u32> {
        let mut m = self.coord_map.borrow_mut();

        let gl_state_opt = self.gl_state.borrow();
        let gl_state = gl_state_opt.as_ref().unwrap();

        match m.get(s) {
            Some(opt_coord) => *opt_coord,
            None => {
                let coord = gl.get_attrib_location(&gl_state.prog, s.into());
                m.insert(s.into(), coord);
                coord
            }
        }
    }

    pub fn set<T>(&self, s: &str, data: T)
    where
        T: 'static + IntoUniformSetter,
    {
        let mut unis = self.pending_uniforms.borrow_mut();
        let mut commited = self.committed_unforms.borrow_mut();

        // Check if the data is committed
        if let Some(cs) = commited.get(s) {
            if *cs == data.to_hash() {
                return;
            }

            commited.remove(s.into());
        }

        unis.insert(s.into(), Box::new(data));
    }

    pub fn commit(&self, gl: &WebGLRenderingContext) {
        let unis = self.pending_uniforms.borrow();
        let mut commited = self.committed_unforms.borrow_mut();

        for (s, data) in &*unis {
            if !commited.contains_key(s) {
                if let Some(u) = self.get_uniform(gl, s) {
                    data.set(gl, &u);
                    commited.insert(s.clone(), data.to_hash());
                }
            }
        }
    }

    fn get_uniform(&self, gl: &WebGLRenderingContext, s: &str) -> Option<Rc<WebGLUniformLocation>> {
        let mut m = self.uniform_map.borrow_mut();
        let gl_state = self.gl_state.borrow();

        match m.get(s) {
            Some(u) => u.as_ref().map(|x| x.clone()),
            None => {
                let uloc = gl.get_uniform_location(&gl_state.as_ref().unwrap().prog, s.into());

                match uloc {
                    None => {
                        m.insert(s.into(), None);
                        None
                    }
                    Some(uloc) => {
                        let p = Rc::new(uloc);
                        m.insert(s.into(), Some(p.clone()));
                        Some(p)
                    }
                }
            }
        }
    }
}

impl Asset for ShaderProgram {
    fn new_from_file<F>(_f: F) -> Rc<Self>
    where
        F: File,
    {
        unimplemented!()
    }
}

impl ShaderProgramGLState {
    pub fn new(gl: &WebGLRenderingContext, vs_code: &str, ps_code: &str) -> ShaderProgramGLState {
        /*================ Shaders ====================*/

        // Create a vertex shader object
        let vert_shader = gl.create_shader(WebGLShaderKind::Vertex);

        // Attach vertex shader source code
        gl.shader_source(&vert_shader, vs_code);

        // Compile the vertex shader
        gl.compile_shader(&vert_shader);

        // Create fragment shader object
        let frag_shader = gl.create_shader(WebGLShaderKind::Fragment);

        // Attach fragment shader source code
        gl.shader_source(&frag_shader, ps_code);

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

        let prog = ShaderProgramGLState {
            prog: shader_program,
        };

        prog
    }
}

// Default vertex shader source code
const DEFAULT_VS: &'static str = include_str!("phong_vs.glsl");
const DEFAULT_FS: &'static str = include_str!("phong_fs.glsl");

const DEFAULT_UI_VS: &'static str = include_str!("ui_vs.glsl");
const DEFAULT_UI_FS: &'static str = include_str!("ui_fs.glsl");
