use engine::render::Texture;
use std::rc::Rc;
use std::ops::Deref;
use engine::render::frame_buffer::FrameBuffer;
use webgl::WebGLRenderingContext;

pub struct RenderTexture(FrameBuffer);

impl Deref for RenderTexture {
    type Target = Rc<Texture>;

    fn deref(&self) -> &Self::Target {
        &self.0.texture
    }
}

impl RenderTexture {
    pub fn new(width: u32, height: u32) -> RenderTexture {
        RenderTexture(FrameBuffer::new(width, height))
    }

    pub fn bind_frame_buffer(&self, gl: &WebGLRenderingContext) {
        self.0.prepare(gl);
        self.0.bind(gl);
    }

    pub fn unbind_frame_buffer(&self, gl: &WebGLRenderingContext) {
        self.0.unbind(gl);
    }

    pub fn as_texture(&self) -> Rc<Texture> {
        self.0.texture.clone()
    }
}
