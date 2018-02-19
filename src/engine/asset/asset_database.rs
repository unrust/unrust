use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use super::{CubeMesh, PlaneMesh, Quad};
use engine::{MeshBuffer, ShaderProgram, Texture, TextureFiltering};

use image;
use image::ImageBuffer;

use super::default_font_bitmap::DEFAULT_FONT_DATA;

pub trait AssetSystem {
    fn new() -> Self
    where
        Self: Sized;

    fn new_program(&self, name: &str) -> Rc<ShaderProgram>;

    fn new_texture(&self, name: &str) -> Rc<Texture>;

    fn new_mesh_buffer(&self, name: &str) -> Rc<MeshBuffer>;
}

pub trait Asset {
    fn new(s: &str) -> Rc<Self>;
}

#[derive(Default)]
pub struct AssetDatabase {
    path: String,
    textures: RefCell<HashMap<String, Rc<Texture>>>,
    mesh_buffers: RefCell<HashMap<String, Rc<MeshBuffer>>>,
    programs: RefCell<HashMap<String, Rc<ShaderProgram>>>,
}

impl AssetSystem for AssetDatabase {
    fn new_program(&self, name: &str) -> Rc<ShaderProgram> {
        let mut a = self.programs.borrow_mut();
        self.new_asset(&mut a, name)
    }

    fn new_texture(&self, name: &str) -> Rc<Texture> {
        let mut a = self.textures.borrow_mut();
        match name {
            name => self.new_asset(&mut a, name),
        }
    }

    fn new_mesh_buffer(&self, name: &str) -> Rc<MeshBuffer> {
        let mut hm = self.mesh_buffers.borrow_mut();
        match hm.get_mut(name) {
            Some(tex) => tex.clone(),
            None => panic!("No asset found."),
        }
    }

    fn new() -> AssetDatabase {
        let mut db = AssetDatabase::default();

        {
            let mut hm = db.mesh_buffers.borrow_mut();
            hm.insert("cube".into(), Rc::new(CubeMesh::new()));
            hm.insert("plane".into(), Rc::new(PlaneMesh::new()));
            hm.insert("screen_quad".into(), Rc::new(Quad::new()));
        }

        {
            let mut hm = db.textures.borrow_mut();
            hm.insert(
                "default_font_bitmap".into(),
                Self::new_default_font_bitmap(),
            );
            hm.insert("default".into(), Self::new_default_texture());
        }

        if cfg!(not(target_arch = "wasm32")) {
            db.path = "static/".into();
        }

        db
    }
}

impl AssetDatabase {
    fn new_asset<R>(&self, hm: &mut HashMap<String, Rc<R>>, name: &str) -> Rc<R>
    where
        R: Asset,
    {
        match hm.get_mut(name) {
            Some(asset) => asset.clone(),
            None => {
                let asset = R::new(&self.get_filename(name));
                hm.insert(name.into(), asset.clone());
                asset
            }
        }
    }

    fn new_default_font_bitmap() -> Rc<Texture> {
        let mut tex = Texture::new_with_image_buffer(ImageBuffer::from_fn(128, 64, |x, y| {
            let cx: u32 = x / 8;
            let cy: u32 = y / 8;
            let c = &DEFAULT_FONT_DATA[(cx + cy * 16) as usize];

            let bx: u8 = (x % 8) as u8;
            let by: u8 = (y % 8) as u8;

            if (c[by as usize] & (1 << bx)) != 0 {
                image::Rgba([0xff, 0xff, 0xff, 0xff])
            } else {
                image::Rgba([0, 0, 0, 0])
            }
        }));

        Rc::get_mut(&mut tex).unwrap().filtering = TextureFiltering::Nearest;

        tex
    }

    fn new_default_texture() -> Rc<Texture> {
        // Construct a new ImageBuffer with the specified width and height.

        // Construct a new by repeated calls to the supplied closure.
        Texture::new_with_image_buffer(ImageBuffer::from_fn(64, 64, |x, y| {
            if (x < 32 && y < 32) || (x > 32 && y > 32) {
                image::Rgba([0xff, 0xff, 0xff, 0xff])
            } else {
                image::Rgba([0, 0, 0, 0xff])
            }
        }))
    }

    pub fn get_filename(&self, name: &str) -> String {
        match name {
            "default" => name.into(),
            "default_font_bitmap" => name.into(),
            "default_ui" => name.into(),
            _ => format!("{}{}", self.path, name),
        }
    }
}
