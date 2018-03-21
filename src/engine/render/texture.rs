use webgl::*;
use webgl;

use image::{RgbImage, RgbaImage};

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use engine::asset::{Asset, AssetResult, AssetSystem, FileFuture, LoadableAsset, Resource};
use std::path::Path;

#[derive(Debug, Copy, Clone)]
pub enum TextureFiltering {
    Nearest,
    Linear,
}

#[derive(Debug, Copy, Clone)]
pub enum TextureWrap {
    Repeat,
    ClampToEdge,
    MirroredRepeat,
}

#[derive(Debug)]
pub enum TextureImage {
    Rgba(RgbaImage),
    Rgb(RgbImage),
}

#[derive(Debug)]
pub enum TextureAttachment {
    Color0,
    Depth,
}

#[derive(Debug)]
enum TextureKind {
    Image(Resource<TextureImage>),
    CubeMap([Resource<TextureImage>; 6]),
    RenderTexture {
        size: (u32, u32),
        attach: TextureAttachment,
    },
}

#[derive(Debug)]
pub struct Texture {
    pub filtering: Cell<TextureFiltering>,
    pub wrap_u: Cell<TextureWrap>,
    pub wrap_v: Cell<TextureWrap>,
    pub wrap_w: Cell<Option<TextureWrap>>,

    gl_state: RefCell<Option<TextureGLState>>,
    kind: TextureKind,
}

pub enum TextureAsset {
    Single(Resource<TextureImage>),
    Cube([Resource<TextureImage>; 6]),
}

impl From<TextureImage> for TextureAsset {
    fn from(img: TextureImage) -> TextureAsset {
        TextureAsset::Single(Resource::new(img))
    }
}

impl Asset for Texture {
    type Resource = TextureAsset;

    fn new_from_resource(r: Self::Resource) -> Rc<Self> {
        return match r {
            TextureAsset::Single(res) => Rc::new(Texture {
                filtering: Cell::new(TextureFiltering::Linear),
                wrap_u: Cell::new(TextureWrap::ClampToEdge),
                wrap_v: Cell::new(TextureWrap::ClampToEdge),
                wrap_w: Cell::new(None),
                gl_state: RefCell::new(None),
                kind: TextureKind::Image(res),
            }),

            TextureAsset::Cube(res) => Rc::new(Texture {
                filtering: Cell::new(TextureFiltering::Linear),
                gl_state: RefCell::new(None),
                kind: TextureKind::CubeMap(res),
                wrap_u: Cell::new(TextureWrap::ClampToEdge),
                wrap_v: Cell::new(TextureWrap::ClampToEdge),
                wrap_w: Cell::new(Some(TextureWrap::ClampToEdge)),
            }),
        };
    }
}

impl LoadableAsset for Texture {
    fn load<T: AssetSystem + Clone + 'static>(
        asys: &T,
        mut files: Vec<FileFuture>,
    ) -> Self::Resource {
        if files.len() == 6 {
            TextureAsset::Cube([
                Self::load_resource::<TextureImage, T>(asys.clone(), files.remove(0)),
                Self::load_resource::<TextureImage, T>(asys.clone(), files.remove(0)),
                Self::load_resource::<TextureImage, T>(asys.clone(), files.remove(0)),
                Self::load_resource::<TextureImage, T>(asys.clone(), files.remove(0)),
                Self::load_resource::<TextureImage, T>(asys.clone(), files.remove(0)),
                Self::load_resource::<TextureImage, T>(asys.clone(), files.remove(0)),
            ])
        } else {
            TextureAsset::Single(Self::load_resource::<TextureImage, T>(
                asys.clone(),
                files.remove(0),
            ))
        }
    }

    fn gather<T: AssetSystem>(asys: &T, fname: &str) -> Vec<FileFuture> {
        let path = Path::new(fname);
        let ext = path.extension();
        let stem = path.file_stem();
        let parent = path.parent();
        let parent = parent.map_or("".to_string(), |p| p.to_str().unwrap().to_string() + "/");

        if ext.is_none() || stem.is_none() {
            return vec![asys.new_file(fname)];
        }

        let ext = ext.unwrap().to_str().unwrap();
        let stem = stem.unwrap().to_str().unwrap();
        let tag = "_cubemap";

        if stem.to_lowercase().ends_with(tag) {
            let f = (&stem[..stem.len() - tag.len()]).to_string();
            return vec![
                asys.new_file(&format!("{}{}_right.{}", &parent, &f, ext)),
                asys.new_file(&format!("{}{}_left.{}", &parent, &f, ext)),
                asys.new_file(&format!("{}{}_top.{}", &parent, &f, ext)),
                asys.new_file(&format!("{}{}_bottom.{}", &parent, &f, ext)),
                asys.new_file(&format!("{}{}_front.{}", &parent, &f, ext)),
                asys.new_file(&format!("{}{}_back.{}", &parent, &f, ext)),
            ];
        }

        vec![asys.new_file(fname)]
    }
}

#[derive(Debug)]
struct TextureGLState {
    tex: WebGLTexture,
    size: (u32, u32),
}

impl Texture {
    pub fn new_render_texture(width: u32, height: u32, attach: TextureAttachment) -> Rc<Self> {
        Rc::new(Texture {
            filtering: Cell::new(TextureFiltering::Linear),
            gl_state: RefCell::new(None),
            wrap_u: Cell::new(TextureWrap::ClampToEdge),
            wrap_v: Cell::new(TextureWrap::ClampToEdge),
            wrap_w: Cell::new(None),
            kind: TextureKind::RenderTexture {
                size: (width, height),
                attach: attach,
            },
        })
    }

    pub fn size(&self) -> Option<(u32, u32)> {
        self.gl_state.borrow().as_ref().map(|ref s| s.size)
    }

    pub fn bind(&self, gl: &WebGLRenderingContext, unit: u32) -> AssetResult<()> {
        self.prepare(gl)?;

        let state_option = self.gl_state.borrow();
        let state = state_option.as_ref().unwrap();

        gl.active_texture(unit);
        match self.kind {
            TextureKind::CubeMap(_) => gl.bind_texture_cube(&state.tex),
            _ => gl.bind_texture(&state.tex),
        }

        Ok(())
    }

    pub fn prepare(&self, gl: &WebGLRenderingContext) -> AssetResult<()> {
        if self.gl_state.borrow().is_some() {
            return Ok(());
        }

        let new_state = Some(texture_bind_buffer(
            gl,
            &self.filtering.get(),
            self.wrap_u.get(),
            self.wrap_v.get(),
            self.wrap_w.get(),
            &self.kind,
        )?);

        self.gl_state.replace(new_state);

        Ok(())
    }
}

fn bind_to_framebuffer(gl: &WebGLRenderingContext, tex: &WebGLTexture, buffer: Buffers) {
    gl.framebuffer_texture2d(
        Buffers::Framebuffer,
        buffer,
        TextureBindPoint::Texture2d,
        tex,
        0,
    );
}

fn unbind_texture(gl: &WebGLRenderingContext, kind: &TextureKind) {
    match kind {
        &TextureKind::Image(_) | &TextureKind::RenderTexture { .. } => {
            gl.unbind_texture();
        }
        &TextureKind::CubeMap(_) => {
            gl.unbind_texture_cube();
        }
    }
}

fn texture_bind_buffer(
    gl: &WebGLRenderingContext,
    texfilter: &TextureFiltering,
    wrap_u: TextureWrap,
    wrap_v: TextureWrap,
    wrap_w: Option<TextureWrap>,
    kind: &TextureKind,
) -> AssetResult<TextureGLState> {
    let mut gl_tex_kind: webgl::TextureKind = webgl::TextureKind::Texture2d;
    let mut force_nearest_filtering = false;

    let (tex, size, has_midmap) = match kind {
        &TextureKind::Image(ref img_res) => {
            let teximg = img_res.try_into()?;

            let tex = gl.create_texture();
            let size: (u32, u32);

            gl.active_texture(0);
            gl.bind_texture(&tex);

            match teximg {
                TextureImage::Rgba(img) => {
                    size = (img.width(), img.height());
                    gl.tex_image2d(
                        TextureBindPoint::Texture2d, // target
                        0,                           // level
                        img.width() as u16,          // width
                        img.height() as u16,         // height
                        PixelFormat::Rgba,           // format
                        PixelType::UnsignedByte,     // type
                        &*img,                       // data
                    );

                    gl.generate_mipmap();
                }
                TextureImage::Rgb(img) => {
                    size = (img.width(), img.height());
                    gl.tex_image2d(
                        TextureBindPoint::Texture2d, // target
                        0,                           // level
                        img.width() as u16,          // width
                        img.height() as u16,         // height
                        PixelFormat::Rgb,            // format
                        PixelType::UnsignedByte,     // type
                        &*img,                       // data
                    );

                    gl.generate_mipmap();
                }
            }

            (tex, size, true)
        }
        &TextureKind::CubeMap(ref img_res) => {
            let mut imgs = Vec::new();
            let mut size: (u32, u32) = (0, 0);

            let bindpoints = [
                TextureBindPoint::TextureCubeMapPositiveX,
                TextureBindPoint::TextureCubeMapNegativeX,
                TextureBindPoint::TextureCubeMapPositiveY,
                TextureBindPoint::TextureCubeMapNegativeY,
                TextureBindPoint::TextureCubeMapPositiveZ,
                TextureBindPoint::TextureCubeMapNegativeZ,
            ];

            // Test if all resources are ready.
            assert!(img_res.len() == 6);
            for res in img_res.iter() {
                res.try_borrow()?;
            }

            for res in img_res.iter() {
                imgs.push(res.try_into()?);
            }

            let tex = gl.create_texture();
            gl.active_texture(0);
            gl.bind_texture_cube(&tex);

            for (i, teximg) in imgs.iter().enumerate() {
                match teximg {
                    &TextureImage::Rgba(ref img) => {
                        size = (img.width(), img.height());
                        gl.tex_image2d(
                            bindpoints[i],           // target
                            0,                       // level
                            img.width() as u16,      // width
                            img.height() as u16,     // height
                            PixelFormat::Rgba,       // format
                            PixelType::UnsignedByte, // type
                            &*img,                   // data
                        );
                    }
                    &TextureImage::Rgb(ref img) => {
                        size = (img.width(), img.height());
                        gl.tex_image2d(
                            bindpoints[i],           // target
                            0,                       // level
                            img.width() as u16,      // width
                            img.height() as u16,     // height
                            PixelFormat::Rgb,        // format
                            PixelType::UnsignedByte, // type
                            &*img,                   // data
                        );
                    }
                }
            }

            gl_tex_kind = webgl::TextureKind::TextureCubeMap;
            gl.generate_mipmap_cube();

            (tex, size, true)
        }

        &TextureKind::RenderTexture { size, ref attach } => {
            let (fmt, data_type) = match attach {
                &TextureAttachment::Color0 => (PixelFormat::Rgba, PixelType::UnsignedByte),
                &TextureAttachment::Depth => {
                    force_nearest_filtering = true;
                    (PixelFormat::DepthComponent, PixelType::UnsignedShort)
                }
            };

            let tex = gl.create_texture();
            gl.active_texture(0);
            gl.bind_texture(&tex);
            gl.tex_image2d(
                TextureBindPoint::Texture2d, // target
                0,                           // level
                size.0 as u16,               // width
                size.1 as u16,               // height
                fmt,                         // format
                data_type,                   // type
                &[],                         // data
            );

            (tex, size, false)
        }
    };

    let mut filtering: (i32, i32) = match texfilter {
        &TextureFiltering::Nearest => (
            TextureMinFilter::Nearest as i32,
            TextureMagFilter::Nearest as i32,
        ),
        _ => {
            if has_midmap {
                (
                    TextureMinFilter::LinearMipmapLinear as i32,
                    TextureMagFilter::Linear as i32,
                )
            } else {
                (
                    TextureMinFilter::Linear as i32,
                    TextureMagFilter::Linear as i32,
                )
            }
        }
    };

    if force_nearest_filtering {
        filtering = (
            TextureMinFilter::Nearest as i32,
            TextureMagFilter::Nearest as i32,
        )
    }

    let to_gl_wrap = |w: TextureWrap| match w {
        TextureWrap::Repeat => webgl::TextureWrap::Repeat as i32,
        TextureWrap::ClampToEdge => webgl::TextureWrap::ClampToEdge as i32,
        TextureWrap::MirroredRepeat => webgl::TextureWrap::MirroredRepeat as i32,
    };

    gl.tex_parameteri(gl_tex_kind, TextureParameter::TextureMinFilter, filtering.0);
    gl.tex_parameteri(gl_tex_kind, TextureParameter::TextureMagFilter, filtering.1);

    gl.tex_parameteri(
        gl_tex_kind,
        TextureParameter::TextureWrapS,
        to_gl_wrap(wrap_u),
    );
    gl.tex_parameteri(
        gl_tex_kind,
        TextureParameter::TextureWrapT,
        to_gl_wrap(wrap_v),
    );

    if let &TextureKind::CubeMap(..) = kind {
        if let Some(wrap_w) = wrap_w {
            gl.tex_parameteri(
                gl_tex_kind,
                TextureParameter::TextureWrapR,
                to_gl_wrap(wrap_w),
            );
        }
    }

    if let &TextureKind::RenderTexture { ref attach, .. } = kind {
        match attach {
            &TextureAttachment::Color0 => bind_to_framebuffer(gl, &tex, Buffers::ColorAttachment0),
            &TextureAttachment::Depth => bind_to_framebuffer(gl, &tex, Buffers::DepthAttachment),
        }
    }

    unbind_texture(gl, kind);

    Ok(TextureGLState { tex, size })
}
