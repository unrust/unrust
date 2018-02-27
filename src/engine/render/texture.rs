use webgl::*;
use image::RgbaImage;

use std::cell::RefCell;
use std::rc::Rc;
use engine::asset::{Asset, AssetError, AssetSystem, FileFuture, LoadableAsset, Resource};

#[derive(Debug)]
pub enum TextureFiltering {
    Nearest,
    Linear,
}

#[derive(Debug)]
enum TextureKind {
    Image(Resource<RgbaImage>),
    RenderTexture { size: (u32, u32) },
}

#[derive(Debug)]
pub struct Texture {
    pub filtering: TextureFiltering,
    gl_state: RefCell<Option<TextureGLState>>,
    kind: TextureKind,
}

impl Asset for Texture {
    type Resource = Resource<RgbaImage>;

    fn new_from_resource(res: Self::Resource) -> Rc<Self> {
        Rc::new(Texture {
            filtering: TextureFiltering::Linear,
            gl_state: RefCell::new(None),
            kind: TextureKind::Image(res),
        })
    }
}

impl LoadableAsset for Texture {
    fn load<T: AssetSystem + Clone + 'static>(
        asys: &T,
        mut files: Vec<FileFuture>,
    ) -> Self::Resource {
        Self::load_resource::<RgbaImage, T>(asys.clone(), files.remove(0))
    }

    fn gather<T: AssetSystem>(asys: &T, fname: &str) -> Vec<FileFuture> {
        vec![asys.new_file(fname)]
    }
}

#[derive(Debug)]
struct TextureGLState {
    tex: WebGLTexture,
}

impl Texture {
    pub fn new_render_texture(width: u32, height: u32) -> Rc<Self> {
        Rc::new(Texture {
            filtering: TextureFiltering::Linear,
            gl_state: RefCell::new(None),
            kind: TextureKind::RenderTexture {
                size: (width, height),
            },
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

        let new_state = Some(texture_bind_buffer(gl, &self.filtering, &self.kind)?);

        self.gl_state.replace(new_state);

        Ok(())
    }
}

fn bind_to_framebuffer(gl: &WebGLRenderingContext, tex: &WebGLTexture) {
    gl.framebuffer_texture2d(
        Buffers::Framebuffer,
        Buffers::ColorAttachment0,
        TextureBindPoint::Texture2d,
        tex,
        0,
    );
}

fn texture_bind_buffer(
    gl: &WebGLRenderingContext,
    texfilter: &TextureFiltering,
    kind: &TextureKind,
) -> Result<TextureGLState, AssetError> {
    let tex = gl.create_texture();

    gl.active_texture(0);
    gl.bind_texture(&tex);

    match kind {
        &TextureKind::Image(ref tex) => {
            let img = tex.try_into()?;

            gl.tex_image2d(
                TextureBindPoint::Texture2d, // target
                0,                           // level
                img.width() as u16,          // width
                img.height() as u16,         // height
                PixelFormat::Rgba,           // format
                DataType::U8,                // type
                &*img,                       // data
            );
        }

        &TextureKind::RenderTexture { size } => {
            gl.tex_image2d(
                TextureBindPoint::Texture2d, // target
                0,                           // level
                size.0 as u16,               // width
                size.1 as u16,               // height
                PixelFormat::Rgba,           // format
                DataType::U8,                // type
                &[],                         // data
            );
        }
    }

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

    if let &TextureKind::RenderTexture { .. } = kind {
        bind_to_framebuffer(gl, &tex);
    }

    gl.unbind_texture();

    Ok(TextureGLState { tex: tex })
}
