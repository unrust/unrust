use webgl::*;
use image::{RgbaImage,ImageBuffer};

use std::cell::RefCell;
use std::rc::Rc;
use engine::asset::{Asset, AssetError, AssetSystem, FileFuture, LoadableAsset, Resource};

pub enum TextureFiltering {
    Nearest,
    Linear,
}

pub struct Texture {
    pub filtering: TextureFiltering,
    gl_state: RefCell<Option<TextureGLState>>,
    img: Resource<RgbaImage>,
}

impl Asset for Texture {
    type Resource = Resource<RgbaImage>;

    fn new_from_resource(res: Self::Resource) -> Rc<Self> {
        Rc::new(Texture {
            filtering: TextureFiltering::Linear,
            img: res,
            gl_state: RefCell::new(None),
        })
    }
}

impl LoadableAsset for Texture {
    fn load<T: AssetSystem>(_asys: &T, mut files: Vec<FileFuture>) -> Self::Resource {
        Self::load_resource::<RgbaImage>(files.remove(0))
    }

    fn gather<T: AssetSystem>(asys: &T, fname: &str) -> Vec<FileFuture> {
        vec![asys.new_file(fname)]
    }
}

struct TextureGLState {
    tex: WebGLTexture,
}

impl Texture {
    pub fn new_empty(width: u32, height: u32) -> Rc<Self> {
        Rc::new(Texture {
            filtering: TextureFiltering::Linear,
            img: Resource::new(ImageBuffer::new(width, height)),
            gl_state: RefCell::new(None),
        })
    }

    pub fn bind(&self, gl: &WebGLRenderingContext, unit: u32, with_framebuffer: bool) -> Result<(), AssetError> {
        self.prepare(gl, with_framebuffer)?;

        let state_option = self.gl_state.borrow();
        let state = state_option.as_ref().unwrap();

        gl.active_texture(unit);
        gl.bind_texture(&state.tex);

        Ok(())
    }

    pub fn prepare(&self, gl: &WebGLRenderingContext, with_framebuffer: bool) -> Result<(), AssetError> {
        if self.gl_state.borrow().is_some() {
            return Ok(());
        }

        let img = self.img.try_into()?;
        self.gl_state
            .replace(Some(texture_bind_buffer(&img, gl, &self.filtering, with_framebuffer)));

        Ok(())
    }


}

fn bind_to_framebuffer(gl: &WebGLRenderingContext, tex: &WebGLTexture) {
    gl.framebuffer_texture2d(Buffers::Framebuffer, Buffers::ColorAttachment0, TextureBindPoint::Texture2d, tex, 0);
}

fn texture_bind_buffer(
    img: &RgbaImage,
    gl: &WebGLRenderingContext,
    texfilter: &TextureFiltering,
    with_framebuffer: bool,
) -> TextureGLState {
    let tex = gl.create_texture();

    gl.active_texture(0);
    gl.bind_texture(&tex);

    gl.tex_image2d(
        TextureBindPoint::Texture2d, // target
        0,                           // level
        img.width() as u16,          // width
        img.height() as u16,         // height
        PixelFormat::Rgba,           // format
        DataType::U8,                // type
        &*img,                       // data
    );

    let filtering: i32 = match texfilter {
        &TextureFiltering::Nearest => TextureMagFilter::Nearest as i32,
        _ => TextureMagFilter::Linear as i32,
    };

    gl.tex_parameteri(TextureParameter::TextureMagFilter, filtering);
    gl.tex_parameteri(TextureParameter::TextureMinFilter, filtering);
    gl.tex_parameteri(
        TextureParameter::TextureWrapS,
        TextureWrap::ClampToEdge as i32,
    );
    gl.tex_parameteri(
        TextureParameter::TextureWrapT,
        TextureWrap::ClampToEdge as i32,
    );

    if with_framebuffer {
        bind_to_framebuffer(gl, &tex);
    }

    gl.unbind_texture();

    TextureGLState { tex: tex }
}
