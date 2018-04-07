use engine::core::Aabb;
use engine::core::ComponentBased;
use engine::render::{Material, MeshBuffer};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Copy, Clone)]
pub struct MeshBound {
    pub aabb: Aabb,
    pub r: f32,
}

impl MeshBound {
    pub fn empty() -> MeshBound {
        MeshBound {
            aabb: Aabb::empty(),
            r: 0.0,
        }
    }

    pub fn local_aabb(&self) -> Aabb {
        self.aabb
    }

    pub fn merge(&mut self, other: &Self) {
        self.aabb.merge(&other.aabb);
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

    pub fn remove_buffer(&mut self, buffer: &Rc<MeshBuffer>) {
        self.surfaces
            .retain(|surface| !Rc::ptr_eq(buffer, &surface.buffer))
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
