use webgl::*;
use image;
use image::RgbaImage;

use std::cell::RefCell;
use std::rc::Rc;
use engine::asset::{Asset, AssetError, AssetSystem, FileFuture, Resource};
use futures::prelude::*;

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
    fn new_from_file<T: AssetSystem>(asys: &T, fname: &str) -> Rc<Self> {
        Texture::new_texture(asys.new_file(fname))
    }
}

struct TextureGLState {
    tex: WebGLTexture,
}

impl Texture {
    fn new_texture(f: FileFuture) -> Rc<Self> {
        let img = f.then(|r| {
            let mut file = r.map_err(|e| AssetError::FileIoError(e))?;
            let buf = file.read_binary().map_err(|_| AssetError::InvalidFormat)?;
            let img = image::load_from_memory(&buf).map_err(|_| AssetError::InvalidFormat)?;
            Ok(img.to_rgba())
        });

        return Rc::new(Texture {
            filtering: TextureFiltering::Linear,
            img: Resource::new_future(img),
            gl_state: RefCell::new(None),
        });
    }

    pub fn new_with_image_buffer(img: RgbaImage) -> Rc<Self> {
        Rc::new(Texture {
            filtering: TextureFiltering::Linear,
            img: Resource::new(img),
            gl_state: RefCell::new(None),
        })
    }

    pub fn bind(&self, gl: &WebGLRenderingContext, unit: u32) -> Result<(), AssetError> {
        self.prepare(gl)?;

        let state_option = self.gl_state.borrow();
        let state = state_option.as_ref().unwrap();

        gl.active_texture(unit);
        gl.bind_texture(&state.tex);

        Ok(())
    }

    pub fn prepare(&self, gl: &WebGLRenderingContext) -> Result<(), AssetError> {
        if self.gl_state.borrow().is_some() {
            return Ok(());
        }

        let img = self.img.try_into()?;
        self.gl_state
            .replace(Some(texture_bind_buffer(&img, gl, &self.filtering)));

        Ok(())
    }
}

fn texture_bind_buffer(
    img: &RgbaImage,
    gl: &WebGLRenderingContext,
    texfilter: &TextureFiltering,
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

    gl.unbind_texture();

    TextureGLState { tex: tex }
}
