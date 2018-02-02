#![feature(nll)]
#![recursion_limit = "512"]

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

use nphysics3d::object::RigidBody;
use na::{Point3, Vector3};
use ncollide::shape::{Cuboid3, Plane3, Shape3};

type Handle<T> = Rc<RefCell<T>>;

fn new_handle<T>(t: T) -> Handle<T> {
    Rc::new(RefCell::new(t))
}

struct Entity {
    go: Handle<GameObject>,
    rb: Handle<RigidBody<f32>>,
}

struct MeshManager {
    meshes: HashMap<&'static str, Rc<Mesh>>,
}

impl MeshManager {
    pub fn new(engine: &Handle<Engine>) -> MeshManager {
        MeshManager {
            meshes: {
                let mut hm: HashMap<&'static str, Rc<Mesh>> = HashMap::new();
                hm.insert("cube", Rc::new(CubeMesh::new(&engine.borrow_mut())));
                hm.insert("plane", Rc::new(PlaneMesh::new(&engine.borrow_mut())));
                hm
            },
        }
    }

    pub fn get(&self, shape: &Shape3<f32>) -> Option<Rc<Mesh>> {
        if let Some(_) = shape.as_shape::<Cuboid3<f32>>() {
            return Some(self.meshes.get("cube").unwrap().clone());
        } else if let Some(_) = shape.as_shape::<Plane3<f32>>() {
            return Some(self.meshes.get("plane").unwrap().clone());
        }

        return None;
    }
}

pub fn main() {
    let size = (800, 600);
    let config = AppConfig::new("Test", size);
    let app = App::new(config);
    {
        let engine = new_handle(Engine::new(&app, size));
        let mesh_mgr = new_handle(MeshManager::new(&engine));

        app.add_control_text();

        let mut camera = Camera::new();

        camera.lookat(
            &Point3::new(0.0, 10.0, 10.0),
            &Point3::new(0.0, 0.0, 0.0),
            &Vector3::new(0.0, 1.0, 0.0),
        );

        let scene = Rc::new(RefCell::new(Scene::new()));

        engine.borrow_mut().main_camera = Some(camera);

        let cubes: Handle<Vec<Entity>> = new_handle(vec![]);

        for rb in scene.borrow_mut().world.rigid_bodies() {
            let mut rbody = rb.borrow();
            let mesh_mgr = &mesh_mgr.borrow();

            let cube = Rc::new(RefCell::new(GameObject {
                transform: *rbody.position(),
                mesh: mesh_mgr.get(rbody.shape().as_ref()).unwrap(),
                shader_program: "default",
            }));

            engine.borrow_mut().add(cube.clone());
            cubes.borrow_mut().push(Entity {
                go: cube,
                rb: rb.clone(),
            });
        }

        let mut fps = FPS::new();
        let mut offset = Box::new(0.0 as f32);

        app.run(move |app: &mut App| {
            fps.step();
            scene.borrow_mut().step();

            {
                let mut engine = engine.borrow_mut();
                let cubes = cubes.clone();

                for evt in app.events.borrow().iter() {
                    match evt {
                        &AppEvent::Click => {
                            let scene = scene.clone();
                            let mesh_mgr = mesh_mgr.clone();

                            let rb = scene.borrow_mut().add_box();
                            let rbody = rb.borrow();
                            let mesh_mgr = &mesh_mgr.borrow();

                            let cube = Rc::new(RefCell::new(GameObject {
                                transform: *rbody.position(),
                                mesh: mesh_mgr.get(rbody.shape().as_ref()).unwrap(),
                                shader_program: "default",
                            }));

                            engine.add(cube.clone());
                            cubes.borrow_mut().push(Entity {
                                go: cube,
                                rb: rb.clone(),
                            })
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

            for cube in cubes.borrow_mut().iter_mut() {
                cube.go.borrow_mut().transform = *cube.rb.borrow().position();
            }

            *offset.as_mut() += 0.01;

            {
                engine.borrow_mut().render();
            }
        });
    }
}
