use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use webgl::{ShaderKind as WebGLShaderKind, WebGLProgram, WebGLRenderingContext,
            WebGLUniformLocation};
use engine::asset::{Asset, AssetResult, AssetSystem, FileFuture, LoadableAsset,
                    Resource};
use engine::render::shader::{ShaderFs, ShaderVs};
use engine::render::uniforms::*;

impl Asset for ShaderProgram {
    type Resource = (Resource<ShaderVs>, Resource<ShaderFs>);

    fn new_from_resource((vs, fs): Self::Resource) -> Rc<ShaderProgram> {
        Rc::new(ShaderProgram {
            gl_state: RefCell::new(None),

            coord_map: Default::default(),
            uniform_map: Default::default(),

            pending_uniforms: Default::default(),
            committed_unforms: Default::default(),

            vs_shader: vs,
            fs_shader: fs,
        })
    }
}

impl LoadableAsset for ShaderProgram {
    fn load<T: AssetSystem + Clone + 'static>(
        asys: &T,
        mut files: Vec<FileFuture>,
    ) -> Self::Resource {
        (
            Self::load_resource::<ShaderVs, T>(asys.clone(), files.remove(0)),
            Self::load_resource::<ShaderFs, T>(asys.clone(), files.remove(0)),
        )
    }

    fn gather<T: AssetSystem>(asys: &T, fname: &str) -> Vec<FileFuture> {
        vec![
            asys.new_file(&format!("{}_vs.glsl", fname)),
            asys.new_file(&format!("{}_fs.glsl", fname)),
        ]
    }
}

#[derive(Debug)]
pub struct ShaderProgramGLState {
    prog: WebGLProgram,
}

#[derive(Debug)]
pub struct ShaderProgram {
    gl_state: RefCell<Option<ShaderProgramGLState>>,

    coord_map: RefCell<HashMap<String, Option<u32>>>,
    uniform_map: RefCell<HashMap<String, Option<Rc<WebGLUniformLocation>>>>,

    vs_shader: Resource<ShaderVs>,
    fs_shader: Resource<ShaderFs>,

    pending_uniforms: RefCell<HashMap<String, Box<UniformAdapter>>>,
    committed_unforms: RefCell<HashMap<String, u64>>,
}

impl ShaderProgram {
    pub fn bind(&self, gl: &WebGLRenderingContext) -> AssetResult<()> {
        self.prepare(gl)?;

        let gl_state = self.gl_state.borrow();
        gl.use_program(&gl_state.as_ref().unwrap().prog);

        // after use, we should clean up the committed uniform state.
        // https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glUniform.xhtml
        self.committed_unforms.borrow_mut().clear();

        Ok(())
    }

    fn prepare(&self, gl: &WebGLRenderingContext) -> AssetResult<()> {
        if self.gl_state.borrow().is_some() {
            return Ok(());
        }

        let vs = self.vs_shader.try_borrow()?;
        let fs = self.fs_shader.try_borrow()?;

        let state = Some(ShaderProgramGLState::new(gl, &vs.code, &fs.code));
        *self.gl_state.borrow_mut() = state;

        Ok(())
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
        T: 'static + UniformAdapter,
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
