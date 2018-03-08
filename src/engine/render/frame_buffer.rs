use webgl::*;

use std::rc::Rc;
use std::cell::RefCell;
use engine::render::{Texture, TextureAttachment};

pub struct FrameBuffer {
    pub texture: Rc<Texture>,
    handle: RefCell<Option<WebGLFrameBuffer>>,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32, attach: TextureAttachment) -> FrameBuffer {
        let texture = Texture::new_render_texture(width, height, attach);
        let handle = RefCell::new(None);
        FrameBuffer { texture, handle }
    }

    fn create_fb(&self, gl: &WebGLRenderingContext) {
        *self.handle.borrow_mut() = Some(gl.create_framebuffer());
    }

    pub fn prepare(&self, gl: &WebGLRenderingContext) {
        if self.handle.borrow().is_some() {
            return;
        }

        self.create_fb(gl);
        self.bind(gl);
        self.texture.bind(gl, 0).unwrap();
        self.unbind(gl);
    }

    pub fn bind(&self, gl: &WebGLRenderingContext) {
        let ho = self.handle.borrow();
        let h = ho.as_ref().unwrap();

        gl.bind_framebuffer(Buffers::Framebuffer, &h);
    }

    pub fn unbind(&self, gl: &WebGLRenderingContext) {
        gl.unbind_framebuffer(Buffers::Framebuffer);
    }
}
