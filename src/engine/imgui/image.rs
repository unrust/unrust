use super::Metric;
use super::instance::ImguiState;
use super::widgets;
use super::widgets::Widget;

use engine::{Asset, GameObject, IEngine, Material, Mesh, MeshBuffer, MeshData, RenderQueue,
             Texture};
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

fn make_quad_mesh_data(ndc_size: (f32, f32)) -> MeshData {
    let w = ndc_size.0;
    let h = ndc_size.1;

    let vertices: Vec<f32> = vec![
            0.0, 0.0, 0.0,     // 0
            0.0, -h, 0.0,    // 1
            w, -h, 0.0,     // 2
            w, 0.0, 0.0       // 3
        ];

    let uvs: Vec<f32> = vec![
            // Top face
            0.0, 1.0,
            0.0, 0.0,
            1.0, 0.0,
            1.0, 1.0,
        ];

    let indices: Vec<u16> = vec![
        0, 1, 2, 0, 2, 3 // Top face
    ];

    MeshData {
        vertices: vertices,
        uvs: Some(uvs),
        normals: None,
        indices: indices,
        tangents: None,
        bitangents: None,
    }
}

fn compute_size_to_ndc(size: &Metric, ssize: &(u32, u32), hidpi: f32) -> (f32, f32) {
    let (x, y) = match size {
        &Metric::Native(px, py) => (px * 2.0, py * 2.0),
        &Metric::Pixel(px, py) => widgets::to_pixel_pos(px, py, ssize, hidpi),
        &Metric::Mixed((ax, ay), (bx, by)) => {
            let vp = widgets::to_pixel_pos(bx, by, ssize, hidpi);
            (ax * 2.0 + vp.0, ay * 2.0 + vp.1)
        }
    };

    return (x, y);
}

#[derive(Debug)]
pub struct ImageRef<T: Debug>(Rc<T>);
impl<T: Debug> PartialEq for ImageRef<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Debug, PartialEq)]
pub enum ImageKind {
    Texture(ImageRef<Texture>),
    Material(ImageRef<Material>),
}

impl From<Rc<Material>> for ImageKind {
    fn from(t: Rc<Material>) -> ImageKind {
        ImageKind::Material(ImageRef(t))
    }
}

impl From<Rc<Texture>> for ImageKind {
    fn from(t: Rc<Texture>) -> ImageKind {
        ImageKind::Texture(ImageRef(t))
    }
}

#[derive(Debug, PartialEq)]
pub struct Image {
    id: u32,
    pos: Metric,
    size: Metric,
    pivot: Metric,
    kind: ImageKind,
}

impl Image {
    pub fn new<T>(id: u32, pos: Metric, size: Metric, state: ImguiState, t: T) -> Widget
    where
        T: Into<ImageKind>,
    {
        Widget::Image(Self {
            id,
            pos,
            size,
            pivot: state.pivot,
            kind: t.into(),
        })
    }

    fn create_material(&self, engine: &mut IEngine) -> Rc<Material> {
        match self.kind {
            ImageKind::Material(ref m) => m.0.clone(),
            ImageKind::Texture(ref t) => {
                let db = engine.asset_system();

                let mut m = Material::new(db.new_program("default_ui"));
                m.render_queue = RenderQueue::UI;
                m.set("uDiffuse", t.0.clone());
                Rc::new(m)
            }
        }
    }

    pub fn bind(
        &self,
        ssize: (u32, u32),
        parent: &GameObject,
        engine: &mut IEngine,
    ) -> Rc<RefCell<GameObject>> {
        let hidpi = engine.hidpi_factor();

        // Mesh Data
        let meshdata = make_quad_mesh_data(compute_size_to_ndc(&self.size, &ssize, hidpi));

        // Material
        let material = self.create_material(engine);

        //Mesh
        let mut mesh = Mesh::new();
        mesh.add_surface(MeshBuffer::new(meshdata), material);

        // Game Object
        let go = engine.new_game_object(parent);
        let mut gomut = go.borrow_mut();

        let mut gtrans = gomut.transform.global();
        gtrans.disp += widgets::compute_translate(
            &self.pos,
            &self.pivot,
            &ssize,
            hidpi,
            &mesh.bounds().unwrap().local_aabb(),
        );
        gomut.transform.set_global(gtrans);
        gomut.add_component(mesh);
        drop(gomut);

        go
    }
}

impl widgets::WidgetBinder for Image {
    fn id(&self) -> u32 {
        self.id
    }

    fn is_same(&self, other: &Widget) -> bool {
        match other {
            &Widget::Image(ref img) => img == self,
            _ => false,
        }
    }
}
