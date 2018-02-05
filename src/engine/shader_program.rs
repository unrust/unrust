use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use webgl::*;

use Asset;

#[derive(Debug)]
pub struct ShaderProgramGLState {
    prog: WebGLProgram,
}

#[derive(Debug, Default)]
pub struct ShaderProgram {
    gl_state: RefCell<Option<ShaderProgramGLState>>,

    coord_map: RefCell<HashMap<&'static str, u32>>,
    uniform_map: RefCell<HashMap<&'static str, Rc<WebGLUniformLocation>>>,
}

impl ShaderProgram {
    pub fn bind(&self, gl: &WebGLRenderingContext) {
        self.prepare(gl);

        self.use_program(gl)
    }

    fn prepare(&self, gl: &WebGLRenderingContext) {
        let is_none = self.gl_state.borrow().is_none();

        if is_none {
            {
                *self.gl_state.borrow_mut() = Some(ShaderProgramGLState::new(gl));
            }

            let pcoord = self.get_coord(gl, "aVertexPosition");
            gl.enable_vertex_attrib_array(pcoord);

            let ncoord = self.get_coord(gl, "aVertexNormal");
            gl.enable_vertex_attrib_array(ncoord);

            let texcoord = self.get_coord(gl, "aTextureCoord");
            gl.enable_vertex_attrib_array(texcoord);
        }
    }

    pub fn get_coord(&self, gl: &WebGLRenderingContext, s: &'static str) -> u32 {
        let mut m = self.coord_map.borrow_mut();

        let gl_state_opt = self.gl_state.borrow();
        let gl_state = gl_state_opt.as_ref().unwrap();

        match m.get(s) {
            Some(coord) => *coord,
            None => {
                let coord = gl.get_attrib_location(&gl_state.prog, s.into()).unwrap();
                m.insert(s.into(), coord);
                coord
            }
        }
    }

    fn use_program(&self, gl: &WebGLRenderingContext) {
        let gl_state = self.gl_state.borrow();
        gl.use_program(&gl_state.as_ref().unwrap().prog);
    }

    pub fn get_uniform(
        &self,
        gl: &WebGLRenderingContext,
        s: &'static str,
    ) -> Rc<WebGLUniformLocation> {
        let mut m = self.uniform_map.borrow_mut();
        let gl_state = self.gl_state.borrow();

        match m.get(s) {
            Some(u) => u.clone(),
            None => {
                let u = Rc::new(gl.get_uniform_location(
                    &gl_state.as_ref().unwrap().prog,
                    s.into(),
                ).unwrap());
                {
                    m.insert(s.into(), u.clone());
                }
                u
            }
        }
    }
}

impl Asset for ShaderProgram {
    fn new(_s: &str) -> Rc<ShaderProgram> {
        Rc::new(Default::default())
    }
}

impl ShaderProgramGLState {
    pub fn new(gl: &WebGLRenderingContext) -> ShaderProgramGLState {
        /*================ Shaders ====================*/

        // Vertex shader source code
        let vert_code = "       
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
        varying vec2 vTextureCoord;
        uniform sampler2D uSampler;

        void main(void) {
            gl_FragColor = vec4(vColor, 1.0) * texture2D(uSampler, vec2(vTextureCoord.s, vTextureCoord.t));
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

        let prog = ShaderProgramGLState {
            prog: shader_program,
        };

        prog
    }
}
