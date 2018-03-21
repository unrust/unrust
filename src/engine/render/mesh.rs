use engine::core::ComponentBased;
use std::rc::Rc;
use std::cell::Cell;
use engine::render::{Material, MeshBuffer};

use na::Vector3;

#[derive(Copy, Clone)]
pub struct MeshBound {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
    pub r: f32,
}

impl MeshBound {
    pub fn empty() -> MeshBound {
        use std::f32::{MAX, MIN};

        MeshBound {
            min: Vector3::new(MAX, MAX, MAX),
            max: Vector3::new(MIN, MIN, MIN),
            r: 0.0,
        }
    }

    pub fn local_aabb(&self) -> (Vector3<f32>, Vector3<f32>) {
        (self.min, self.max)
    }

    pub fn merge(&mut self, other: &Self) {
        self.min[0] = self.min[0].min(other.min[0]);
        self.min[1] = self.min[1].min(other.min[1]);
        self.min[2] = self.min[2].min(other.min[2]);

        self.max[0] = self.max[0].max(other.max[0]);
        self.max[1] = self.max[1].max(other.max[1]);
        self.max[2] = self.max[2].max(other.max[2]);

        self.r = self.r.max(other.r);
    }
}

pub struct MeshSurface {
    pub buffer: Rc<MeshBuffer>,
    pub material: Rc<Material>,
}

pub struct Mesh {
    pub surfaces: Vec<Rc<MeshSurface>>,
    pub mesh_bounds: Cell<Option<MeshBound>>,
}

impl ComponentBased for Mesh {}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
            surfaces: Vec::new(),
            mesh_bounds: Cell::new(None),
        }
    }

    pub fn add_surface<U, T>(&mut self, buffer: U, material: T)
    where
        U: Into<Rc<MeshBuffer>>,
        T: Into<Rc<Material>>,
    {
        self.surfaces.push(Rc::new(MeshSurface {
            buffer: buffer.into(),
            material: material.into(),
        }));
    }

    /// bounds return (vmin, vmax)
    pub fn bounds(&self) -> Option<MeshBound> {
        if let Some(_) = self.mesh_bounds.get() {
            return self.mesh_bounds.get();
        }

        if self.surfaces
            .iter()
            .find(|s| s.buffer.bounds().is_none())
            .is_some()
        {
            return None;
        }

        let bb = self.surfaces.iter().fold(MeshBound::empty(), |mut acc, s| {
            s.buffer.bounds().map(|b| acc.merge(&b));

            acc
        });

        self.mesh_bounds.set(Some(bb));
        self.mesh_bounds.get()
    }
}
