use webgl::*;

use std::rc::Rc;
use engine::render::Texture;

pub struct Framebuffer {
    pub texture:Rc<Texture>,
    handle: WebGLFrameBuffer,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, gl: &WebGLRenderingContext)-> Framebuffer {
        let texture = Texture::new_empty(width, height);
        let handle = gl.create_framebuffer();

        Framebuffer {
            texture,
            handle,
        }
    }
    pub fn prepare(&self, gl: &WebGLRenderingContext) {
        self.bind(gl);
        self.texture.bind(gl, 0, true).unwrap();
    }
    pub fn bind(&self, gl: &WebGLRenderingContext) {
        gl.bind_framebuffer(Buffers::Framebuffer, &self.handle);
    }
    pub fn unbind(&self, gl: &WebGLRenderingContext) {
        gl.unbind_framebuffer(Buffers::Framebuffer);
    }
}
