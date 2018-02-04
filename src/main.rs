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
use na::Isometry3;

use nphysics3d::object::RigidBody;
use na::{Point3, Vector3};
use ncollide::shape::{Cuboid3, Plane3, Shape3};

type Handle<T> = Rc<RefCell<T>>;

fn new_handle<T>(t: T) -> Handle<T> {
    Rc::new(RefCell::new(t))
}

struct PhysicObject(Handle<RigidBody<f32>>);
impl PhysicObject {
    fn get_phy_transform(&self) -> Isometry3<f32> {
        self.0.borrow().position().clone()
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

fn add_object(
    mesh_mgr: Handle<MeshManager>,
    engine: Handle<Engine>,
    rb: Handle<RigidBody<f32>>,
) -> Handle<GameObject> {
    let mesh_mgr = &mesh_mgr.borrow();

    let mut eng = engine.borrow_mut();
    let go = eng.new_gameobject(rb.borrow().position());
    {
        let mut c = go.borrow_mut();

        c.add_component(mesh_mgr.get(rb.borrow().shape().as_ref()).unwrap());
        c.add_component(Material::new_component("default"));
        c.add_component(Component::new(PhysicObject(rb)));
    }

    go
}

pub fn main() {
    let size = (800, 600);
    let config = AppConfig::new("Test", size);
    let app = App::new(config);
    {
        let engine = new_handle(Engine::new(&app, size));
        let mesh_mgr = new_handle(MeshManager::new());

        app.add_control_text();

        let mut camera = Camera::new();

        camera.lookat(
            &Point3::new(0.0, 10.0, 10.0),
            &Point3::new(0.0, 0.0, 0.0),
            &Vector3::new(0.0, 1.0, 0.0),
        );

        let scene = Rc::new(RefCell::new(Scene::new()));

        engine.borrow_mut().main_camera = Some(camera);

        let mut cubes: Vec<Handle<GameObject>> = vec![];

        for rb in scene.borrow_mut().world.rigid_bodies() {
            let go = add_object(mesh_mgr.clone(), engine.clone(), rb.clone());
            cubes.push(go);
        }

        let mut fps = FPS::new();
        let mut offset = Box::new(0.0 as f32);

        app.run(move |app: &mut App| {
            fps.step();
            scene.borrow_mut().step();

            {
                for evt in app.events.borrow().iter() {
                    match evt {
                        &AppEvent::Click => {
                            let scene = scene.clone();
                            let rb = scene.borrow_mut().add_box();
                            let go = add_object(mesh_mgr.clone(), engine.clone(), rb.clone());

                            cubes.push(go)
                        }
                    }
                }
            }

            // cam.lookat(
            //     &Point3::new(10.0 * offset.sin(), 10.0, 10.0 * offset.cos()),
            //     &Point3::new(0.0, 0.0, 0.0),
            //     &Vector3::new(0.0, 1.0, 0.0),
            // );

            {
                let mut engine = engine.borrow_mut();
                let cam = engine.main_camera.as_mut().unwrap();
                cam.lookat(
                    &Point3::new(-30.0, 30.0, -30.0),
                    &Point3::new(0.0, 0.0, 0.0),
                    &Vector3::new(0.0, 1.0, 0.0),
                );
            }

            let get_pb_tran = |o: &GameObject| {
                let (rb, _) = o.get_component_by_type::<PhysicObject>().unwrap();
                rb.get_phy_transform()
            };

            for go in cubes.iter() {
                let tran = get_pb_tran(&go.borrow());
                go.borrow_mut().transform = tran;
            }

            *offset.as_mut() += 0.01;

            {
                engine.borrow_mut().render();
            }
        });
    }
}
