use webgl::*;
use image;
use image::RgbaImage;

use std::cell::RefCell;
use std::rc::Rc;
use engine::asset::{Asset, FileFuture, FileIoError};
use futures::prelude::*;

pub enum TextureFiltering {
    Nearest,
    Linear,
}

enum ImageResource {
    Empty,
    Image(RgbaImage),
    Future(Box<Future<Item = RgbaImage, Error = FileIoError>>),
}

pub struct Texture {
    pub filtering: TextureFiltering,
    gl_state: RefCell<Option<TextureGLState>>,
    img: RefCell<ImageResource>,
}

impl Asset for Texture {
    fn new_from_file(f: FileFuture) -> Rc<Self> {
        Texture::new_texture(f)
    }
}

struct TextureGLState {
    tex: WebGLTexture,
}

impl Texture {
    fn new_texture(f: FileFuture) -> Rc<Self> {
        let img = f.and_then(|mut file| {
            let buf = file.read_binary()?;
            let img = image::load_from_memory(&buf).map_err(|_| FileIoError::InvalidFormat)?;
            Ok(img.to_rgba())
        });

        return Rc::new(Texture {
            filtering: TextureFiltering::Linear,
            img: RefCell::new(ImageResource::Future(Box::new(img))),
            gl_state: RefCell::new(None),
        });
    }

    pub fn new_with_image_buffer(img: RgbaImage) -> Rc<Self> {
        Rc::new(Texture {
            filtering: TextureFiltering::Linear,
            img: RefCell::new(ImageResource::Image(img)),
            gl_state: RefCell::new(None),
        })
    }

    pub fn bind(&self, gl: &WebGLRenderingContext, unit: u32) -> Result<(), String> {
        self.prepare(gl)?;

        let state_option = self.gl_state.borrow();
        let state = state_option.as_ref().unwrap();

        gl.active_texture(unit);
        gl.bind_texture(&state.tex);

        Ok(())
    }

    fn prepare_img(&self) -> Result<RgbaImage, String> {
        if let &mut ImageResource::Future(ref mut f) = &mut *self.img.borrow_mut() {
            return match f.poll() {
                Err(_) => Err("Fail to know".to_string()),
                Ok(Async::NotReady) => Err("Not ready".to_string()),
                Ok(Async::Ready(i)) => Ok(i),
            };
        }

        if let ImageResource::Image(i) = self.img.replace(ImageResource::Empty) {
            return Ok(i);
        }

        unreachable!()
    }

    pub fn prepare(&self, gl: &WebGLRenderingContext) -> Result<(), String> {
        if self.gl_state.borrow().is_some() {
            return Ok(());
        }

        let img = self.prepare_img()?;
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
