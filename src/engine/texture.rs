use webgl::*;
use image;
use image::{ImageBuffer, RgbaImage};
use Engine;
use std::cell::RefCell;
use std::rc::Rc;
use ShaderProgram;
use Asset;

use uni_app::{File, FileSystem, IoError};
use std::io::ErrorKind;

pub struct Texture {
    gl_state: RefCell<Option<TextureGLState>>,

    img: RefCell<Option<RgbaImage>>,
    file: Option<RefCell<File>>,
}

impl Asset for Texture {
    fn new(s: &str) -> Rc<Self> {
        match s {
            "default" => Texture::new_default(),
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
                img: RefCell::new(None),
                file: Some(RefCell::new(f)),
                gl_state: RefCell::new(None),
            }));
        }

        let buf = f.read_binary()?;

        match image::load_from_memory(&buf) {
            Ok(img) => Ok(Rc::new(Texture {
                img: RefCell::new(Some(img.to_rgba())),
                gl_state: RefCell::new(None),
                file: None,
            })),
            Err(err) => Err(IoError::new(ErrorKind::Other, err)),
        }
    }

    fn new_default() -> Rc<Self> {
        // Construct a new ImageBuffer with the specified width and height.

        // Construct a new by repeated calls to the supplied closure.
        let img = ImageBuffer::from_fn(64, 64, |x, y| {
            if (x < 32 && y < 32) || (x > 32 && y > 32) {
                image::Rgba([0xff, 0xff, 0xff, 0xff])
            } else {
                image::Rgba([0, 0, 0, 0xff])
            }
        });

        Rc::new(Texture {
            img: RefCell::new(Some(img)),
            gl_state: RefCell::new(None),
            file: None,
        })
    }

    pub fn bind(&self, engine: &Engine, program: &ShaderProgram) -> bool {
        if !self.prepare(engine) {
            return false;
        }

        let gl = &engine.gl;
        let state_option = self.gl_state.borrow();
        let state = state_option.as_ref().unwrap();

        gl.bind_texture(&state.tex);

        let scoord = program.get_uniform(gl, "uSampler");
        gl.uniform_1i(&scoord, 0);

        true
    }

    fn need_file(&self) -> bool {
        match self.file {
            None => false,
            Some(_) => true,
        }
    }

    pub fn prepare(&self, engine: &Engine) -> bool {
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

            self.gl_state
                .replace(Some(texture_bind_buffer(&img.as_ref().unwrap(), engine)));
        }

        true
    }
}

fn texture_bind_buffer(img: &RgbaImage, engine: &Engine) -> TextureGLState {
    let gl = &engine.gl;

    let tex = gl.create_texture();

    gl.bind_texture(&tex);

    let data: &[u8] = &*img;

    println!("{}", img.width() as u16);
    println!("{}", img.height() as u16);
    println!("{}", data.len());

    gl.tex_image2d(
        TextureBindPoint::Texture2d, // target
        0,                           // level
        img.width() as u16,          // width
        img.height() as u16,         // height
        PixelFormat::Rgba,           // format
        DataType::U8,                // type
        &*img,                       // data
    );

    gl.tex_parameteri(
        TextureParameter::TextureMagFilter,
        TextureMagFilter::Linear as i32,
    );

    gl.tex_parameteri(
        TextureParameter::TextureMinFilter,
        TextureMagFilter::Linear as i32,
    );

    gl.unbind_texture();

    TextureGLState { tex: tex }
}
