#![feature(nll)]
#![recursion_limit = "512"]
#![feature(integer_atomics)]

/* common */
extern crate nalgebra as na;
extern crate ncollide;
extern crate nphysics3d;
extern crate uni_app;
extern crate webgl;

mod boxes_vee;
mod engine;

use boxes_vee::*;
use engine::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use uni_app::*;
use std::sync::Arc;
use std::ops::Deref;
use na::Isometry3;

use nphysics3d::object::RigidBody;
use na::{Point3, Vector3};
use ncollide::shape::{Cuboid3, Plane3, Shape3};

type Handle<T> = Rc<RefCell<T>>;

// Physic Object Component
struct PhysicObject(Handle<RigidBody<f32>>);
impl PhysicObject {
    fn get_phy_transform(&self) -> Isometry3<f32> {
        *self.0.borrow().position()
    }
}

impl ComponentBased for PhysicObject {}

struct MeshManager {
    meshes: HashMap<&'static str, Arc<Component>>,
}

impl MeshManager {
    pub fn new() -> MeshManager {
        MeshManager {
            meshes: {
                let mut hm: HashMap<&'static str, Arc<Component>> = HashMap::new();
                hm.insert("cube", PrimitiveMesh::new_cube_component());
                hm.insert("plane", PrimitiveMesh::new_plane_component());

                hm
            },
        }
    }

    pub fn get(&self, shape: &Shape3<f32>) -> Option<Arc<Component>> {
        if let Some(_) = shape.as_shape::<Cuboid3<f32>>() {
            return Some(self.meshes.get("cube").unwrap().clone());
        } else if let Some(_) = shape.as_shape::<Plane3<f32>>() {
            return Some(self.meshes.get("plane").unwrap().clone());
        }

        return None;
    }
}

struct GameObjectList(Vec<Handle<GameObject>>);

impl GameObjectList {
    fn add_object(
        &mut self,
        mesh_mgr: &MeshManager,
        engine: &mut Engine,
        rb: Handle<RigidBody<f32>>,
    ) -> Handle<GameObject> {
        let go = engine.new_gameobject(rb.borrow().position());
        {
            let mut go_mut = go.borrow_mut();

            go_mut.add_component(mesh_mgr.get(rb.borrow().shape().as_ref()).unwrap());
            go_mut.add_component(Material::new_component("default"));
            go_mut.add_component(Component::new(PhysicObject(rb)));
        }

        self.0.push(go.clone());
        go
    }
}

impl Deref for GameObjectList {
    type Target = Vec<Handle<GameObject>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn main() {
    let size = (800, 600);
    let config = AppConfig::new("Test", size);
    let app = App::new(config);
    {
        let mut engine = Engine::new(&app, size);
        let mesh_mgr = MeshManager::new();
        let mut scene = Scene::new();

        app.add_control_text();

        let mut camera = Camera::new();

        camera.lookat(
            &Point3::new(0.0, 10.0, 10.0),
            &Point3::new(0.0, 0.0, 0.0),
            &Vector3::new(0.0, 1.0, 0.0),
        );

        engine.main_camera = Some(camera);

        let mut objects: GameObjectList = GameObjectList(vec![]);

        for rb in scene.world.rigid_bodies() {
            objects.add_object(&mesh_mgr, &mut engine, rb.clone());
        }

        let mut fps = FPS::new();

        app.run(move |app: &mut App| {
            fps.step();
            scene.step();

            // Handle Events
            {
                for evt in app.events.borrow().iter() {
                    match evt {
                        &AppEvent::Click => {
                            objects.add_object(&mesh_mgr, &mut engine, scene.add_box());
                        }
                    }
                }
            }

            // Update Camera
            {
                let cam = engine.main_camera.as_mut().unwrap();
                cam.lookat(
                    &Point3::new(-30.0, 30.0, -30.0),
                    &Point3::new(0.0, 0.0, 0.0),
                    &Vector3::new(0.0, 1.0, 0.0),
                );
            }

            // Update Transforms by physic object
            {
                let get_pb_tran = |o: &GameObject| {
                    let (rb, _) = o.get_component_by_type::<PhysicObject>().unwrap();
                    rb.get_phy_transform()
                };

                for go in objects.iter() {
                    let tran = get_pb_tran(&go.borrow());
                    go.borrow_mut().transform = tran;
                }
            }

            // Render
            {
                engine.render();
            }
        });
    }
}
