use engine::core::GameObject;
use engine::render::{Material, Mesh};
use engine::engine::IEngine;
use engine::render::Texture;

use super::Metric;

use std::fmt::Debug;
use engine::render::{MeshBuffer, MeshData};
use engine::asset::Asset;

use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

use na::{Translation3, Vector3};

pub trait Widget: Debug {
    fn id(&self) -> u32;
    fn bind(
        &self,
        ssize: (u32, u32),
        parent: &GameObject,
        engine: &mut IEngine,
    ) -> Rc<RefCell<GameObject>>;

    fn is_same(&self, other: &Widget) -> bool;
    fn as_any(&self) -> &Any;
}

impl PartialEq for Widget {
    fn eq(&self, other: &Widget) -> bool {
        self.is_same(other)
    }
}

fn make_text_mesh_data(s: &str, size: (u32, u32)) -> MeshData {
    let mut vertices = vec![];
    let mut uvs = vec![];
    let mut indices = vec![];

    let icw = 8.0 / 128.0;
    let ich = 8.0 / 64.0;

    let mut i = 0;
    let nrow = 128 / 8;

    let gw = ((8 as f32) / size.0 as f32) * 2.0;
    let gh = ((8 as f32) / size.1 as f32) * 2.0;
    let mut base_y = 0.0;

    let lines: Vec<&str> = s.split('\n').collect();

    for line in lines.into_iter() {
        for (cidx, c) in line.chars().enumerate() {
            let mut c: u8 = c as u8;
            if c >= 128 {
                c = 128;
            }

            let g_row = (c / nrow) as f32;
            let g_col = (c % nrow) as f32;

            let gx = (cidx as f32) * gw;

            vertices.append(&mut vec![
                gx + 0.0, // 0
                base_y,
                0.0,
                gx + 0.0, // 1
                base_y - gh,
                0.0,
                gx + gw, // 2
                base_y - gh,
                0.0,
                gx + gw, // 3
                base_y,
                0.0,
            ]);

            uvs.append(&mut vec![
                g_col * icw + 0.0, // 0
                g_row * ich,
                g_col * icw + 0.0, // 1
                g_row * ich + ich,
                g_col * icw + icw, // 2
                g_row * ich + ich,
                g_col * icw + icw, // 3
                g_row * ich,
            ]);

            indices.append(&mut vec![
                i * 4,
                i * 4 + 1,
                i * 4 + 2,
                i * 4 + 0,
                i * 4 + 2,
                i * 4 + 3, // Top face
            ]);

            i += 1;
        }

        base_y -= gh * 2.0;
    }

    MeshData {
        vertices: vertices,
        uvs: Some(uvs),
        normals: None,
        indices: indices,
    }
}

fn make_quad_mesh_data(size: (f32, f32)) -> MeshData {
    let w = size.0;
    let h = size.1;

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
    }
}

fn compute_size_to_ndc(size: &Metric, ssize: &(u32, u32)) -> (f32, f32) {
    let (x, y) = match size {
        &Metric::Native(px, py) => (px * 2.0, py * 2.0),
        &Metric::Pixel(px, py) => to_pixel_pos(px, py, ssize),
        &Metric::Mixed((ax, ay), (bx, by)) => {
            let vp = to_pixel_pos(bx, by, ssize);
            (ax * 2.0 + vp.0, ay * 2.0 + vp.1)
        }
    };

    return (x, y);
}

fn compute_translate(
    pos: &Metric,
    pivot: &Metric,
    ssize: &(u32, u32),
    bounds: (Vector3<f32>, Vector3<f32>),
) -> Translation3<f32> {
    let w = bounds.1.x - bounds.0.x;
    let h = bounds.1.y - bounds.0.y;

    let (x, y) = match pos {
        &Metric::Native(px, py) => (px * 2.0, py * 2.0),
        &Metric::Pixel(px, py) => to_pixel_pos(px, py, ssize),
        &Metric::Mixed((ax, ay), (bx, by)) => {
            let vp = to_pixel_pos(bx, by, ssize);
            (ax * 2.0 + vp.0, ay * 2.0 + vp.1)
        }
    };

    let (offsetx, offsety) = match pivot {
        &Metric::Native(px, py) => (px * w, py * h),
        _ => unreachable!(),
    };

    Translation3::new(x - 1.0 - offsetx, y * -1.0 + 1.0 + offsety, 0.0)
}

fn to_pixel_pos(px: f32, py: f32, ssize: &(u32, u32)) -> (f32, f32) {
    (((px * 2.0) / (ssize.0 as f32), (py * 2.0) / (ssize.1 as f32)))
}

#[derive(Debug, PartialEq)]
pub struct Label {
    id: u32,
    pos: Metric,
    pivot: Metric,
    s: String,
}

impl Label {
    pub fn new(id: u32, pos: Metric, pivot: Metric, s: String) -> Label {
        Self {
            id: id,
            pos: pos,
            pivot: pivot,
            s: s,
        }
    }
}

impl Widget for Label {
    fn id(&self) -> u32 {
        self.id
    }

    fn bind(
        &self,
        ssize: (u32, u32),
        parent: &GameObject,
        engine: &mut IEngine,
    ) -> Rc<RefCell<GameObject>> {
        let go = engine.new_game_object(parent);
        let db = engine.asset_system();

        {
            let mut gomut = go.borrow_mut();
            let meshdata = make_text_mesh_data(&self.s, ssize);

            let mut mesh = Mesh::new();
            let mut material = Material::new(db.new_program("default_ui"));
            material.set("uDiffuse", db.new_texture("default_font_bitmap"));

            mesh.add_surface(MeshBuffer::new(meshdata), Rc::new(material));

            gomut.transform.append_translation_mut(&compute_translate(
                &self.pos,
                &self.pivot,
                &ssize,
                mesh.bounds().unwrap(),
            ));

            gomut.add_component(mesh);
        }

        go
    }

    fn is_same(&self, other: &Widget) -> bool {
        match other.as_any().downcast_ref::<Label>() {
            Some(o) => o == self,
            _ => false,
        }
    }

    fn as_any(&self) -> &Any {
        self
    }
}

#[derive(Debug)]
struct ImageTexture(Rc<Texture>);

impl PartialEq for ImageTexture {
    fn eq(&self, other: &ImageTexture) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    fn ne(&self, other: &ImageTexture) -> bool {
        !Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct Image {
    id: u32,
    pos: Metric,
    size: Metric,
    pivot: Metric,
    texture: ImageTexture,
}

impl Image {
    pub fn new(id: u32, pos: Metric, size: Metric, pivot: Metric, texture: Rc<Texture>) -> Image {
        Self {
            id: id,
            pos: pos,
            size: size,
            pivot: pivot,
            texture: ImageTexture(texture),
        }
    }
}

impl Widget for Image {
    fn id(&self) -> u32 {
        self.id
    }

    fn bind(
        &self,
        ssize: (u32, u32),
        parent: &GameObject,
        engine: &mut IEngine,
    ) -> Rc<RefCell<GameObject>> {
        let go = engine.new_game_object(parent);
        let db = engine.asset_system();

        {
            let mut gomut = go.borrow_mut();
            let meshdata = make_quad_mesh_data(compute_size_to_ndc(&self.size, &ssize));

            let mut mesh = Mesh::new();
            let mut material = Material::new(db.new_program("default_ui"));
            material.set("uDiffuse", self.texture.0.clone());

            mesh.add_surface(MeshBuffer::new(meshdata), Rc::new(material));

            gomut.transform.append_translation_mut(&compute_translate(
                &self.pos,
                &self.pivot,
                &ssize,
                mesh.bounds().unwrap(),
            ));

            gomut.add_component(mesh);
        }

        go
    }

    fn is_same(&self, other: &Widget) -> bool {
        match other.as_any().downcast_ref::<Image>() {
            Some(o) => o == self,
            _ => false,
        }
    }

    fn as_any(&self) -> &Any {
        self
    }
}
