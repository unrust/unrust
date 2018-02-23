use engine::core::GameObject;
use engine::render::{Material, MaterialParam, Mesh};
use engine::engine::IEngine;

use super::Metric;

use std::fmt::Debug;
use engine::render::MeshBuffer;

use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::collections::HashMap;

use na::{Translation3, Vector3};

pub trait Widget: Send + Sync + Debug {
    fn id(&self) -> u32;
    fn bind(&self, ssize: (u32, u32), engine: &mut IEngine) -> Rc<RefCell<GameObject>>;

    fn is_same(&self, other: &Widget) -> bool;
    fn as_any(&self) -> &Any;
}

impl PartialEq for Widget {
    fn eq(&self, other: &Widget) -> bool {
        self.is_same(other)
    }
}

#[derive(Debug, PartialEq)]
pub struct Label {
    id: u32,
    pos: Metric,
    pivot: Metric,
    s: String,
}

fn make_text_mesh_buffer(s: &str, size: (u32, u32)) -> MeshBuffer {
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

    let mut m = MeshBuffer::default();
    m.vertices = vertices;
    m.uvs = Some(uvs);
    m.normals = None;
    m.indices = indices;
    m
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

fn to_pixel_pos(px: f32, py: f32, ssize: &(u32, u32)) -> (f32, f32) {
    (((px * 2.0) / (ssize.0 as f32), (py * 2.0) / (ssize.1 as f32)))
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

impl Widget for Label {
    fn id(&self) -> u32 {
        self.id
    }

    fn bind(&self, ssize: (u32, u32), engine: &mut IEngine) -> Rc<RefCell<GameObject>> {
        let go = engine.new_gameobject();
        let db = engine.asset_system();

        {
            let mut gomut = go.borrow_mut();
            let mesh = Mesh::new(Rc::new(make_text_mesh_buffer(&self.s, ssize)));
            gomut.transform.append_translation_mut(&compute_translate(
                &self.pos,
                &self.pivot,
                &ssize,
                mesh.mesh_buffer.bounds(),
            ));

            gomut.add_component(mesh);

            let mut textures = HashMap::new();
            textures.insert(
                "uDiffuse".to_string(),
                MaterialParam::Texture(db.new_texture("default_font_bitmap")),
            );

            gomut.add_component(Material::new(db.new_program("default_ui"), textures));
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
