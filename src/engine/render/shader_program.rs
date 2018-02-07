use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use webgl::*;

use engine::Asset;

#[derive(Debug)]
pub struct ShaderProgramGLState {
    prog: WebGLProgram,
}

#[derive(Debug, Default)]
pub struct ShaderProgram {
    gl_state: RefCell<Option<ShaderProgramGLState>>,

    coord_map: RefCell<HashMap<&'static str, Option<u32>>>,
    uniform_map: RefCell<HashMap<&'static str, Option<Rc<WebGLUniformLocation>>>>,

    vs_shader: String,
    ps_shader: String,
}

impl ShaderProgram {
    pub fn new_default() -> ShaderProgram {
        Self::new(

        // Vertex shader source code
        "       
        attribute vec3 aVertexPosition;
        attribute vec3 aVertexNormal;
        attribute vec2 aTextureCoord;

        uniform mat4 uMVMatrix;
        uniform mat4 uPMatrix;
        uniform mat4 uNMatrix;
        varying vec3 vColor;

        varying vec2 vTextureCoord;
        
        void main(void) {
            gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 1.0);

            vec3 uLightingDirection = normalize(vec3(1.0, -1.0, 0.3));
        
            vec4 transformedNormal = uNMatrix * vec4(aVertexNormal, 1.0);
            float directionalLightWeighting = max(dot(transformedNormal.xyz, -uLightingDirection), 0.0);
        
            vColor = vec3(1.0, 1.0, 1.0) * directionalLightWeighting;

            vTextureCoord = aTextureCoord;
        }    
        ",

        //fragment shader source code
        "
        precision mediump float;

        varying vec3 vColor;
        varying vec2 vTextureCoord;
        uniform sampler2D uSampler;

        void main(void) {
            gl_FragColor = vec4(vColor, 1.0) * texture2D(uSampler, vec2(vTextureCoord.s, vTextureCoord.t));
        }
        ")
    }

    pub fn new_default_screen() -> ShaderProgram {
        Self::new(
            // Vertex shader source code
            "       
            attribute vec3 aVertexPosition;
            attribute vec2 aTextureCoord;
            varying vec2 vTextureCoord;
            
            void main(void) {
                gl_Position = vec4(aVertexPosition, 1.0);        
                vTextureCoord = aTextureCoord;
            }    
            ",
            //fragment shader source code
            "precision mediump float;

            varying vec3 vColor;
            varying vec2 vTextureCoord;
            uniform sampler2D uSampler;

            void main(void) {
                gl_FragColor = texture2D(uSampler, vec2(vTextureCoord.s, vTextureCoord.t));
            }
            ",
        )
    }

    pub fn new(vs: &str, ps: &str) -> ShaderProgram {
        let mut program: ShaderProgram = Default::default();

        // Vertex shader source code
        program.vs_shader = vs.into();

        //fragment shader source code
        program.ps_shader = ps.into();

        program
    }

    pub fn bind(&self, gl: &WebGLRenderingContext) {
        self.prepare(gl);

        let gl_state = self.gl_state.borrow();
        gl.use_program(&gl_state.as_ref().unwrap().prog);
    }

    fn prepare(&self, gl: &WebGLRenderingContext) {
        let is_none = self.gl_state.borrow().is_none();

        if is_none {
            {
                let state = Some(ShaderProgramGLState::new(
                    gl,
                    &self.vs_shader,
                    &self.ps_shader,
                ));
                *self.gl_state.borrow_mut() = state;
            }

            if let Some(pcoord) = self.get_coord(gl, "aVertexPosition") {
                gl.enable_vertex_attrib_array(pcoord);
            }

            if let Some(ncoord) = self.get_coord(gl, "aVertexNormal") {
                gl.enable_vertex_attrib_array(ncoord);
            }

            if let Some(texcoord) = self.get_coord(gl, "aTextureCoord") {
                gl.enable_vertex_attrib_array(texcoord);
            }
        }
    }

    pub fn get_coord(&self, gl: &WebGLRenderingContext, s: &'static str) -> Option<u32> {
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

    pub fn get_uniform(
        &self,
        gl: &WebGLRenderingContext,
        s: &'static str,
    ) -> Option<Rc<WebGLUniformLocation>> {
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
    fn new(s: &str) -> Rc<ShaderProgram> {
        match s {
            "default_screen" => Rc::new(ShaderProgram::new_default_screen()),
            _ => Rc::new(ShaderProgram::new_default()),
        }
    }
}

impl ShaderProgramGLState {
    pub fn new(gl: &WebGLRenderingContext, vs_code: &str, ps_code: &str) -> ShaderProgramGLState {
        /*================ Shaders ====================*/

        // Create a vertex shader object
        let vert_shader = gl.create_shader(ShaderKind::Vertex);

        // Attach vertex shader source code
        gl.shader_source(&vert_shader, vs_code);

        // Compile the vertex shader
        gl.compile_shader(&vert_shader);

        // Create fragment shader object
        let frag_shader = gl.create_shader(ShaderKind::Fragment);

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
