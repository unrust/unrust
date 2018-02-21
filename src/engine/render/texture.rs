use webgl::*;
use image;
use image::RgbaImage;

use std::cell::RefCell;
use std::rc::Rc;
use engine::asset::{Asset, File, FileIoError};

pub enum TextureFiltering {
    Nearest,
    Linear,
}

pub struct Texture {
    pub filtering: TextureFiltering,
    gl_state: RefCell<Option<TextureGLState>>,
    img: RefCell<Option<RgbaImage>>,
    file: Option<RefCell<Box<File>>>,
}

impl Asset for Texture {
    fn new_from_file<F>(f: F) -> Rc<Self>
    where
        F: File + 'static,
    {
        let fname = f.name();
        Texture::new_texture(f).expect(&format!("Cannot open file: {}", fname))
    }
}

struct TextureGLState {
    tex: WebGLTexture,
}

impl Texture {
    fn new_texture<F>(mut f: F) -> Result<Rc<Self>, FileIoError>
    where
        F: File + 'static,
    {
        if !f.is_ready() {
            return Ok(Rc::new(Texture {
                filtering: TextureFiltering::Linear,
                img: RefCell::new(None),
                file: Some(RefCell::new(Box::new(f))),
                gl_state: RefCell::new(None),
            }));
        }

        let buf = f.read_binary()?;
        let img = image::load_from_memory(&buf).map_err(|_| FileIoError::InvalidFormat)?;

        Ok(Rc::new(Texture {
            filtering: TextureFiltering::Linear,
            img: RefCell::new(Some(img.to_rgba())),
            gl_state: RefCell::new(None),
            file: None,
        }))
    }

    pub fn new_with_image_buffer(img: RgbaImage) -> Rc<Self> {
        Rc::new(Texture {
            filtering: TextureFiltering::Linear,
            img: RefCell::new(Some(img)),
            gl_state: RefCell::new(None),
            file: None,
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

    fn need_file(&self) -> bool {
        match self.file {
            None => false,
            Some(_) => true,
        }
    }

    pub fn prepare(&self, gl: &WebGLRenderingContext) -> Result<(), String> {
        if self.gl_state.borrow().is_some() {
            return Ok(());
        }

        if self.need_file() {
            let mut file = self.file.as_ref().unwrap().borrow_mut();
            if !file.is_ready() {
                return Err("read file delay.".to_string());
            }

            let buf = file.read_binary()
                .map_err(|_| String::from("read file delay."))?;
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
