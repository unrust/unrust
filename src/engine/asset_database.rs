use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::collections::HashMap;

use Texture;
use ShaderProgram;

use Component;
use PrimitiveMesh;
use Quad;

pub trait Asset {
    fn new(s: &str) -> Rc<Self>;
}

#[derive(Default)]
pub struct AssetDatabase<'a> {
    path: &'static str,
    textures: RefCell<HashMap<&'a str, Rc<Texture>>>,
    meshes: RefCell<HashMap<&'a str, Arc<Component>>>,
    programs: RefCell<HashMap<&'a str, Rc<ShaderProgram>>>,
}

impl<'a> AssetDatabase<'a> {
    pub fn new_asset<R>(&self, hm: &mut HashMap<&'a str, Rc<R>>, name: &'a str) -> Rc<R>
    where
        R: Asset,
    {
        match hm.get_mut(name) {
            Some(asset) => asset.clone(),
            None => {
                let asset = R::new(&self.get_filename(name));
                hm.insert(name, asset.clone());
                asset
            }
        }
    }

    pub fn new_program(&self, name: &'a str) -> Rc<ShaderProgram> {
        let mut a = self.programs.borrow_mut();
        self.new_asset(&mut a, name)
    }

    pub fn new_texture(&self, name: &'a str) -> Rc<Texture> {
        let mut a = self.textures.borrow_mut();
        self.new_asset(&mut a, name)
    }

    pub fn get_filename(&self, name: &'a str) -> String {
        match name {
            "default" => name.into(),
            "default_font_bitmap" => name.into(),
            "default_screen" => name.into(),
            _ => format!("{}{}", self.path, name),
        }
    }

    pub fn new_mesh(&self, name: &'a str) -> Arc<Component> {
        let mut hm = self.meshes.borrow_mut();
        match hm.get_mut(name) {
            Some(tex) => tex.clone(),
            None => panic!("No asset found."),
        }
    }

    pub fn new() -> AssetDatabase<'a> {
        let mut db = AssetDatabase::default();

        db.meshes = RefCell::new({
            let mut hm = HashMap::new();
            hm.insert("cube", PrimitiveMesh::new_cube_component());
            hm.insert("plane", PrimitiveMesh::new_plane_component());
            hm.insert("screen_quad", Quad::new_quad_component());
            hm
        });

        if cfg!(not(target_arch = "wasm32")) {
            db.path = "static/";
        }

        db
    }
}
