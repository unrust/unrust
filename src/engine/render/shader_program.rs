use engine::asset::{Asset, AssetResult, AssetSystem, FileFuture, LoadableAsset, Resource};
use engine::render::shader::{ShaderFs, ShaderVs};
use engine::render::uniforms::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use webgl::{ShaderKind as WebGLShaderKind, WebGLProgram, WebGLRenderingContext};

use std::borrow::Cow;

use uni_app;

pub enum ShaderAttrib {
    Position = 0,
    UV0 = 1,
    Normal = 2,
    Tangent = 3,
    Bitangent = 4,
}

impl Asset for ShaderProgram {
    type Resource = (Resource<ShaderVs>, Resource<ShaderFs>);

    fn new_from_resource((vs, fs): Self::Resource) -> Rc<ShaderProgram> {
        Rc::new(ShaderProgram {
            gl_state: RefCell::new(None),

            coord_map: Default::default(),
            uniform_cache: Default::default(),

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

    vs_shader: Resource<ShaderVs>,
    fs_shader: Resource<ShaderFs>,

    uniform_cache: UniformCache,
}

impl ShaderProgram {
    pub fn bind(&self, gl: &WebGLRenderingContext) -> AssetResult<()> {
        self.prepare(gl)?;

        let gl_state = self.gl_state.borrow();
        gl.use_program(&gl_state.as_ref().unwrap().prog);

        // after use, we should clean up the committed uniform state.
        // https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glUniform.xhtml
        //self.uniform_cache.clear();

        Ok(())
    }

    fn prepare(&self, gl: &WebGLRenderingContext) -> AssetResult<()> {
        if self.gl_state.borrow().is_some() {
            return Ok(());
        }

        let vs = self.vs_shader.try_borrow()?;
        let fs = self.fs_shader.try_borrow()?;

        let state = Some(ShaderProgramGLState::new(gl, &vs, &fs));
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

    pub fn set<T, S>(&self, s: S, data: T)
    where
        T: Into<UniformAdapter>,
        S: Into<Cow<'static, str>>,
    {
        self.uniform_cache.set(s, data);
    }

    pub fn commit(&self, gl: &WebGLRenderingContext) {
        self.gl_state.borrow().as_ref().map(|ref gl_state| {
            self.uniform_cache.commit(gl, &gl_state.prog);
        });
    }
}

impl ShaderProgramGLState {
    pub fn new(
        gl: &WebGLRenderingContext,
        vs_unit: &ShaderVs,
        fs_unit: &ShaderFs,
    ) -> ShaderProgramGLState {
        /*================ Shaders ====================*/

        // Create a vertex shader object
        let vert_shader = gl.create_shader(WebGLShaderKind::Vertex);

        // Attach vertex shader source code
        gl.shader_source(&vert_shader, &vs_unit.code.as_string());

        // Compile the vertex shader
        uni_app::App::print(format!("Compiling shader file : {}\n", vs_unit.filename));
        gl.compile_shader(&vert_shader);

        // Create fragment shader object
        let frag_shader = gl.create_shader(WebGLShaderKind::Fragment);

        // Attach fragment shader source code
        gl.shader_source(&frag_shader, &fs_unit.code.as_string());

        // Compile the fragmentt shader
        uni_app::App::print(format!("Compiling shader file : {}\n", fs_unit.filename));
        gl.compile_shader(&frag_shader);

        // Create a shader program object to store
        // the combined shader program
        let shader_program = gl.create_program();

        // Attach a vertex shader
        gl.attach_shader(&shader_program, &vert_shader);

        // Attach a fragment shader
        gl.attach_shader(&shader_program, &frag_shader);

        // We bind the position to 0
        // see: https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices
        //
        //
        // > Always have vertex attrib 0 array enabled. If you draw with vertex attrib 0 array disabled,
        // you will force the browser to do complicated emulation when running on desktop OpenGL (e.g. on Mac OSX).
        // This is because in desktop OpenGL, nothing gets drawn if vertex attrib 0 is not array-enabled.
        // You can use bindAttribLocation() to force a vertex attribute to use location 0,
        // and use enableVertexAttribArray() to make it array-enabled.
        gl.bind_attrib_location(
            &shader_program,
            "aVertexPosition",
            ShaderAttrib::Position as _,
        );
        gl.bind_attrib_location(&shader_program, "aTextureCoord", ShaderAttrib::UV0 as _);
        gl.bind_attrib_location(&shader_program, "aVertexNormal", ShaderAttrib::Normal as _);
        gl.bind_attrib_location(
            &shader_program,
            "aVertexTangent",
            ShaderAttrib::Tangent as _,
        );
        gl.bind_attrib_location(
            &shader_program,
            "aVertexBitangent",
            ShaderAttrib::Bitangent as _,
        );

        // Link both the programs
        gl.link_program(&shader_program);

        let prog = ShaderProgramGLState {
            prog: shader_program,
        };

        prog
    }
}
