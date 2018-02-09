use engine::core::{Component, GameObject};
use engine::render::{Material, Mesh};

use engine::engine::IEngine;

use std::fmt::Debug;
use engine::render::MeshBuffer;

use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

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
    x: f32,
    y: f32,
    s: String,
}

impl Label {
    pub fn new(id: u32, x: f32, y: f32, s: String) -> Label {
        Self {
            id: id,
            x: x,
            y: y,
            s: s,
        }
    }

    pub fn mesh(&self, size: (u32, u32)) -> MeshBuffer {
        let mut vertices = vec![];
        let mut uvs = vec![];
        let mut indices = vec![];

        let icw = 8.0 / 128.0;
        let ich = 8.0 / 64.0;

        let mut i = 0;
        let nrow = 128 / 8;

        let mut gw = ((8 as f32) / size.0 as f32) * 2.0;
        let mut gh = ((8 as f32) / size.1 as f32) * 2.0;

        // Scale the font 2x
        gw *= 2.0;
        gh *= 2.0;

        for c in self.s.chars() {
            let mut c: u8 = c as u8;
            if c >= 128 {
                c = 128;
            }

            let g_row = (c / nrow) as f32;
            let g_col = (c % nrow) as f32;

            let gx = (i as f32) * gw;

            vertices.append(&mut vec![
                gx + 0.0, // 0
                0.0,
                0.0,
                gx + 0.0, // 1
                -gh,
                0.0,
                gx + gw, // 2
                -gh,
                0.0,
                gx + gw, // 3
                0.0,
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

        MeshBuffer {
            vertices: vertices,
            uvs: Some(uvs),
            normals: None,
            indices: indices,
        }
    }
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
            // go.transform
            //     .append_translation_mut(&Translation3::new(0.0, 0.2, 0.0));

            gomut.add_component(Component::new(Mesh::new(self.mesh(ssize))));
            gomut.add_component(Material::new_component(
                db.new_program("default_screen"),
                db.new_texture("default_font_bitmap"),
            ));
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
