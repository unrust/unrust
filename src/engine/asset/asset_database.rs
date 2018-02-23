use std::rc::Rc;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;

use engine::asset::{CubeMesh, PlaneMesh, Quad};
use engine::asset::default_font_bitmap::DEFAULT_FONT_DATA;
use engine::asset::fs;

use engine::{MeshBuffer, ShaderProgram, Texture, TextureFiltering};
use futures::{Async, Future};
use std::mem;
use std::fmt::Debug;
use std::fmt;

use image;
use image::ImageBuffer;

impl<T> Debug for ResourceKind<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ResourceKind::Consumed => write!(f, "ResourceKind::Consumed"),
            &ResourceKind::Data(ref t) => write!(f, "ResourceKind::Data({:?})", *t),
            &ResourceKind::Future(_) => write!(f, "ResourceKind::Future"),
        }
    }
}

enum ResourceKind<T: Debug> {
    Consumed,
    Data(T),
    Future(Box<Future<Item = T, Error = AssetError>>),
}

impl<T: Debug> ResourceKind<T> {
    fn try_into_data(self) -> Option<T> {
        match self {
            ResourceKind::Data(d) => Some(d),
            _ => None,
        }
    }

    fn try_as_data(&self) -> Option<&T> {
        match self {
            &ResourceKind::Data(ref d) => Some(d),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Resource<T: Debug>(RefCell<ResourceKind<T>>);

impl<T: Debug> Resource<T> {
    pub fn new_future<FT>(f: FT) -> Self
    where
        FT: Future<Item = T, Error = AssetError> + 'static,
    {
        Resource(RefCell::new(ResourceKind::Future(Box::new(f))))
    }

    pub fn new(f: T) -> Self {
        Resource(RefCell::new(ResourceKind::Data(f)))
    }

    pub fn try_into(&self) -> Result<T, AssetError> {
        match &mut *self.0.borrow_mut() {
            &mut ResourceKind::Future(ref mut f) => {
                return match f.poll() {
                    Err(e) => Err(e),
                    Ok(Async::NotReady) => Err(AssetError::NotReady),
                    Ok(Async::Ready(i)) => Ok(i),
                };
            }

            img @ &mut ResourceKind::Data(_) => {
                let r = mem::replace(img, ResourceKind::Consumed);
                Ok(r.try_into_data().unwrap())
            }

            _ => unreachable!(),
        }
    }

    pub fn try_borrow(&self) -> Result<Ref<T>, AssetError> {
        let mut data = None;

        if let &mut ResourceKind::Future(ref mut f) = &mut *self.0.borrow_mut() {
            match f.poll() {
                Err(e) => return Err(e),
                Ok(Async::NotReady) => return Err(AssetError::NotReady),
                Ok(Async::Ready(i)) => {
                    data = Some(i);
                }
            }
        }

        if let Some(i) = data {
            let kind: &mut ResourceKind<T> = &mut self.0.borrow_mut();
            mem::replace(kind, ResourceKind::Data(i));
        }

        let b0 = self.0.borrow();
        return Ok(Ref::map(b0, |t| t.try_as_data().unwrap()));
    }
}

#[derive(Debug, Clone)]
pub enum AssetError {
    NotReady,
    InvalidFormat,
    FileIoError(fs::FileIoError),
}

pub trait AssetSystem {
    fn new() -> Self
    where
        Self: Sized;

    fn new_file(&self, name: &str) -> fs::FileFuture;

    fn new_program(&self, name: &str) -> Rc<ShaderProgram>;

    fn new_texture(&self, name: &str) -> Rc<Texture>;

    fn new_mesh_buffer(&self, name: &str) -> Rc<MeshBuffer>;

    fn reset(&mut self);
}

pub trait Asset {
    fn new_from_file<T: AssetSystem>(asys: &T, fname: &str) -> Rc<Self>;
}

pub struct AssetDatabase<FS, F>
where
    FS: fs::FileSystem<File = F>,
    F: fs::File,
{
    fs: FS,
    path: String,
    textures: RefCell<HashMap<String, Rc<Texture>>>,
    mesh_buffers: RefCell<HashMap<String, Rc<MeshBuffer>>>,
    programs: RefCell<HashMap<String, Rc<ShaderProgram>>>,
}

impl<FS, F> AssetSystem for AssetDatabase<FS, F>
where
    FS: fs::FileSystem<File = F>,
    F: fs::File + 'static,
{
    fn new_file(&self, name: &str) -> fs::FileFuture {
        self.fs.open(&self.get_filename(name))
    }

    fn new_program(&self, name: &str) -> Rc<ShaderProgram> {
        let mut a = self.programs.borrow_mut();
        self.new_asset(&mut a, name)
    }

    fn new_texture(&self, name: &str) -> Rc<Texture> {
        let mut a = self.textures.borrow_mut();
        self.new_asset(&mut a, name)
    }

    fn new_mesh_buffer(&self, name: &str) -> Rc<MeshBuffer> {
        let mut a = self.mesh_buffers.borrow_mut();
        self.new_asset(&mut a, name)
    }

    fn reset(&mut self) {
        self.textures.borrow_mut().clear();
        self.mesh_buffers.borrow_mut().clear();
        self.programs.borrow_mut().clear();

        self.setup();
    }

    fn new() -> AssetDatabase<FS, F> {
        let mut db = AssetDatabase {
            fs: FS::default(),
            path: String::default(),
            textures: RefCell::new(HashMap::new()),
            mesh_buffers: RefCell::new(HashMap::new()),
            programs: RefCell::new(HashMap::new()),
        };

        db.setup();

        if cfg!(not(target_arch = "wasm32")) {
            db.path = "static/".into();
        }

        db
    }
}

impl<FS, F> AssetDatabase<FS, F>
where
    FS: fs::FileSystem<File = F>,
    F: fs::File + 'static,
{
    fn new_asset<R>(&self, hm: &mut HashMap<String, Rc<R>>, name: &str) -> Rc<R>
    where
        R: Asset,
    {
        match hm.get(name) {
            Some(asset) => asset.clone(),
            None => {
                let asset = R::new_from_file(self, name);
                hm.insert(name.into(), asset.clone());
                asset
            }
        }
    }

    fn setup(&mut self) {
        {
            let mut hm = self.mesh_buffers.borrow_mut();
            hm.insert("cube".into(), Rc::new(CubeMesh::new()));
            hm.insert("plane".into(), Rc::new(PlaneMesh::new()));
            hm.insert("screen_quad".into(), Rc::new(Quad::new()));
        }

        {
            let mut hm = self.textures.borrow_mut();
            hm.insert(
                "default_font_bitmap".into(),
                Self::new_default_font_bitmap(),
            );
            hm.insert("default".into(), Self::new_default_texture());
        }

        {
            let mut hm = self.programs.borrow_mut();
            hm.insert("default".into(), Rc::new(ShaderProgram::new_default()));
            hm.insert(
                "default_ui".into(),
                Rc::new(ShaderProgram::new_default_ui()),
            );
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
        format!("{}{}", self.path, name)
    }
}
