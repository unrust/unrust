use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use engine::asset::{CubeMesh, PlaneMesh, QuadMesh, SkyboxMesh};
use engine::asset::default_font_bitmap::DEFAULT_FONT_DATA;
use engine::asset::fs;
use engine::asset::loader;
use engine::asset::loader::Loadable;
use engine::asset::Resource;

use engine::{MeshBuffer, ShaderFs, ShaderProgram, ShaderVs, Texture, TextureFiltering,
             TextureImage};
use std::fmt::Debug;
use std::ops::Deref;
use futures::{Async, Future};
use std::boxed::FnBox;

use image;
use image::ImageBuffer;

#[derive(Debug)]
pub enum AssetError {
    NotReady,
    ReadBufferFail(String),
    InvalidFormat {
        path: String,
        len: usize,
        reason: String,
    },
    FileIoError(fs::FileIoError),
}

pub type AssetResult<T> = Result<T, AssetError>;

type PrefabHandler = Box<FnBox(AssetResult<loader::Prefab>)>;

pub trait AssetSystem {
    fn new() -> Self
    where
        Self: Sized;

    fn new_file(&self, name: &str) -> fs::FileFuture;

    fn new_program(&self, name: &str) -> Rc<ShaderProgram>;

    fn new_texture(&self, name: &str) -> Rc<Texture>;

    fn new_mesh_buffer(&self, name: &str) -> Rc<MeshBuffer>;

    fn new_prefab(&self, name: &str, f: PrefabHandler);

    fn reset(&mut self);

    fn step(&mut self);

    fn loading_files(&self) -> Vec<String>;
}

pub trait Asset {
    type Resource;

    fn new<T>(r: T) -> Rc<Self>
    where
        T: Into<Self::Resource>,
    {
        Asset::new_from_resource(r.into())
    }

    fn new_from_resource(r: Self::Resource) -> Rc<Self>;
}

pub trait LoadableAsset: Asset {
    fn load<T: AssetSystem + Clone + 'static>(
        asys: &T,
        files: Vec<fs::FileFuture>,
    ) -> Self::Resource;

    fn gather<T: AssetSystem>(asys: &T, fname: &str) -> Vec<fs::FileFuture>;

    fn load_resource<U, A>(asys: A, f: fs::FileFuture) -> Resource<U>
    where
        U: loader::Loadable + Debug + 'static,
        A: AssetSystem + Clone + 'static,
    {
        Resource::new_future(U::load_future(asys, f))
    }
}

type PrefabFuture = Box<Future<Item = loader::Prefab, Error = AssetError>>;

pub struct AssetDatabaseContext<FS> {
    fs: FS,
    path: String,
    textures: RefCell<HashMap<String, Rc<Texture>>>,
    mesh_buffers: RefCell<HashMap<String, Rc<MeshBuffer>>>,
    programs: RefCell<HashMap<String, Rc<ShaderProgram>>>,

    pending_prefabs: RefCell<Vec<(PrefabHandler, PrefabFuture)>>,
}

pub struct AssetDatabase<FS, F>
where
    FS: fs::FileSystem<File = F>,
    F: fs::File,
{
    context: Rc<AssetDatabaseContext<FS>>,
}

impl<FS, F> Deref for AssetDatabase<FS, F>
where
    FS: fs::FileSystem<File = F>,
    F: fs::File,
{
    type Target = AssetDatabaseContext<FS>;

    fn deref(&self) -> &Self::Target {
        self.context.as_ref()
    }
}

impl<FS, F> Clone for AssetDatabase<FS, F>
where
    FS: fs::FileSystem<File = F>,
    F: fs::File,
{
    fn clone(&self) -> Self {
        AssetDatabase {
            context: self.context.clone(),
        }
    }
}

impl<FS, F> AssetSystem for AssetDatabase<FS, F>
where
    FS: fs::FileSystem<File = F> + 'static,
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

    fn new_prefab(&self, name: &str, f: PrefabHandler) {
        let prefab = loader::Prefab::load_future(self.clone(), self.new_file(name));
        self.pending_prefabs.borrow_mut().push((f, prefab));
    }

    fn new() -> AssetDatabase<FS, F> {
        let mut db = AssetDatabase {
            context: Rc::new(AssetDatabaseContext {
                fs: FS::default(),
                path: String::default(),
                textures: RefCell::new(HashMap::new()),
                mesh_buffers: RefCell::new(HashMap::new()),
                programs: RefCell::new(HashMap::new()),
                pending_prefabs: RefCell::new(Vec::new()),
            }),
        };

        db.setup();
        db
    }

    fn step(&mut self) {
        let pending_prefabs = self.pending_prefabs
            .borrow_mut()
            .drain(0..)
            .collect::<Vec<_>>();

        let new_pending = pending_prefabs
            .into_iter()
            .filter_map(|(f, mut prefab)| match prefab.poll() {
                Err(e) => {
                    f(Err(e));
                    None
                }
                Ok(Async::NotReady) => Some((f, prefab)),
                Ok(Async::Ready(i)) => {
                    f(Ok(i));
                    None
                }
            })
            .collect();

        *self.pending_prefabs.borrow_mut() = new_pending;
    }

    fn loading_files(&self) -> Vec<String> {
        self.fs.loading_files()
    }
}

impl<FS, F> AssetDatabase<FS, F>
where
    FS: fs::FileSystem<File = F> + 'static,
    F: fs::File + 'static,
{
    fn new_asset<R>(&self, hm: &mut HashMap<String, Rc<R>>, name: &str) -> Rc<R>
    where
        R: LoadableAsset,
    {
        match hm.get(name) {
            Some(asset) => asset.clone(),
            None => {
                let asset = R::new(R::load(self, R::gather(self, name)));
                hm.insert(name.into(), asset.clone());
                asset
            }
        }
    }

    fn setup(&mut self) {
        {
            let mut hm = self.mesh_buffers.borrow_mut();
            hm.insert("cube".into(), MeshBuffer::new(CubeMesh::new()));
            hm.insert("plane".into(), MeshBuffer::new(PlaneMesh::new()));
            hm.insert("screen_quad".into(), MeshBuffer::new(QuadMesh::new()));
            hm.insert("skybox".into(), MeshBuffer::new(SkyboxMesh::new()));
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
            hm.insert("default".into(), Self::new_default_program());
            hm.insert("default_ui".into(), Self::new_default_ui_program());
        }
    }

    fn new_default_font_bitmap() -> Rc<Texture> {
        let mut tex = Texture::new(TextureImage::Rgba(ImageBuffer::from_fn(128, 64, |x, y| {
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
        })));

        Rc::get_mut(&mut tex).unwrap().filtering = TextureFiltering::Nearest;

        tex
    }

    fn new_default_texture() -> Rc<Texture> {
        // Construct a new ImageBuffer with the specified width and height.

        // Construct a new by repeated calls to the supplied closure.
        Texture::new(TextureImage::Rgba(ImageBuffer::from_fn(64, 64, |x, y| {
            if (x < 32 && y < 32) || (x > 32 && y > 32) {
                image::Rgba([0xff, 0xff, 0xff, 0xff])
            } else {
                image::Rgba([0, 0, 0, 0xff])
            }
        })))
    }

    pub fn new_default_program() -> Rc<ShaderProgram> {
        let vs = ShaderVs::new("phong_vs.glsl", DEFAULT_VS);
        let fs = ShaderFs::new("phong_fs.glsl", DEFAULT_FS);

        ShaderProgram::new((Resource::new(vs), Resource::new(fs)))
    }

    pub fn new_default_ui_program() -> Rc<ShaderProgram> {
        let vs = ShaderVs::new("ui_vs.glsl", DEFAULT_UI_VS);
        let fs = ShaderFs::new("ui_fs.glsl", DEFAULT_UI_FS);

        ShaderProgram::new((Resource::new(vs), Resource::new(fs)))
    }

    pub fn get_filename(&self, name: &str) -> String {
        format!("{}{}", self.path, name)
    }
}

// Default vertex shader source code
const DEFAULT_VS: &'static str = include_str!("phong_vs.glsl");
const DEFAULT_FS: &'static str = include_str!("phong_fs.glsl");

const DEFAULT_UI_VS: &'static str = include_str!("ui_vs.glsl");
const DEFAULT_UI_FS: &'static str = include_str!("ui_fs.glsl");
