use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use webgl::*;

pub struct ShaderProgram {
    prog: WebGLProgram,

    coord_map: RefCell<HashMap<&'static str, u32>>,
    uniform_map: RefCell<HashMap<&'static str, Rc<WebGLUniformLocation>>>,
}

impl ShaderProgram {
    pub fn new(gl: &WebGLRenderingContext) -> ShaderProgram {
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

        let prog = ShaderProgram {
            prog: shader_program,
            coord_map: RefCell::new(HashMap::new()),
            uniform_map: RefCell::new(HashMap::new()),
        };

        let pcoord = prog.get_coord(gl, "aVertexPosition");
        gl.enable_vertex_attrib_array(pcoord);

        let ncoord = prog.get_coord(gl, "aVertexNormal");
        gl.enable_vertex_attrib_array(ncoord);

        let texcoord = prog.get_coord(gl, "aTextureCoord");
        gl.enable_vertex_attrib_array(texcoord);

        prog.get_uniform(gl, "uSampler");

        prog
    }

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

    pub fn use_program(&self, gl: &WebGLRenderingContext) {
        gl.use_program(&self.prog);
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
