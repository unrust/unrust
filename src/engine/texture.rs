use webgl::*;
use image;
use image::{ImageBuffer, RgbaImage};
use Engine;
use std::cell::RefCell;
use std::rc::Rc;
use ShaderProgram;
use Asset;

pub struct Texture {
    img: RgbaImage,
    gl_state: RefCell<Option<TextureGLState>>,
}

impl Asset for Texture {
    fn new(_s: &str) -> Rc<Self> {
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
            img: img,
            gl_state: RefCell::new(None),
        })
    }
}

struct TextureGLState {
    tex: WebGLTexture,
}

impl Texture {
    pub fn bind(&self, engine: &Engine, program: &ShaderProgram) {
        self.prepare(engine);

        let gl = &engine.gl;
        let state_option = self.gl_state.borrow();
        let state = state_option.as_ref().unwrap();

        gl.bind_texture(&state.tex);
        let program_gl_state_b = program.gl_state.borrow();
        let program_gl_state = program_gl_state_b.as_ref().unwrap();

        let scoord = program_gl_state.get_uniform(gl, "uSampler");
        gl.uniform_1i(&scoord, 0);
    }

    pub fn prepare(&self, engine: &Engine) {
        if self.gl_state.borrow().is_none() {
            self.gl_state
                .replace(Some(texture_bind_buffer(&self.img, engine)));
        }
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
