use engine::Asset;

use webgl::*;
use image;
use image::RgbaImage;

use std::cell::RefCell;
use std::rc::Rc;

use uni_app::{File, FileSystem, IoError};
use std::io::ErrorKind;

pub enum TextureFiltering {
    Nearest,
    Linear,
}

pub struct Texture {
    pub filtering: TextureFiltering,
    gl_state: RefCell<Option<TextureGLState>>,
    img: RefCell<Option<RgbaImage>>,
    file: Option<RefCell<File>>,
}

impl Asset for Texture {
    fn new(s: &str) -> Rc<Self> {
        match s {
            filename => {
                Texture::new_texture(filename).expect(&format!("Cannot open file: {:?}", filename))
            }
        }
    }
}

struct TextureGLState {
    tex: WebGLTexture,
}

impl Texture {
    fn new_texture(filename: &str) -> Result<Rc<Self>, IoError> {
        let mut f = FileSystem::open(filename)?;
        if !f.is_ready() {
            return Ok(Rc::new(Texture {
                filtering: TextureFiltering::Linear,
                img: RefCell::new(None),
                file: Some(RefCell::new(f)),
                gl_state: RefCell::new(None),
            }));
        }

        let buf = f.read_binary()?;

        match image::load_from_memory(&buf) {
            Ok(img) => Ok(Rc::new(Texture {
                filtering: TextureFiltering::Linear,
                img: RefCell::new(Some(img.to_rgba())),
                gl_state: RefCell::new(None),
                file: None,
            })),
            Err(err) => Err(IoError::new(ErrorKind::Other, err)),
        }
    }

    pub fn new_with_image_buffer(img: RgbaImage) -> Rc<Self> {
        Rc::new(Texture {
            filtering: TextureFiltering::Linear,
            img: RefCell::new(Some(img)),
            gl_state: RefCell::new(None),
            file: None,
        })
    }

    pub fn bind(&self, gl: &WebGLRenderingContext, unit: u32) -> bool {
        if !self.prepare(gl) {
            return false;
        }

        let state_option = self.gl_state.borrow();
        let state = state_option.as_ref().unwrap();

        gl.active_texture(unit);
        gl.bind_texture(&state.tex);

        true
    }

    fn need_file(&self) -> bool {
        match self.file {
            None => false,
            Some(_) => true,
        }
    }

    pub fn prepare(&self, gl: &WebGLRenderingContext) -> bool {
        if self.gl_state.borrow().is_some() {
            return true;
        }

        if self.need_file() {
            let mut file = self.file.as_ref().unwrap().borrow_mut();
            if !file.is_ready() {
                return false;
            }

            let buf = file.read_binary()
                .expect("Cannot read binary from file delayed.");
            let img = image::load_from_memory(&buf).expect("Cannot load image from file delayed.");
            *self.img.borrow_mut() = Some(img.to_rgba());
        }

        if self.gl_state.borrow().is_none() {
            let img = self.img.borrow();

            self.gl_state.replace(Some(texture_bind_buffer(
                &img.as_ref().unwrap(),
                gl,
                &self.filtering,
            )));
        }

        true
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
