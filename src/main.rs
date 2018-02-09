#![feature(nll)]
#![recursion_limit = "512"]
#![feature(integer_atomics)]

/* common */
extern crate nalgebra as na;
extern crate ncollide;
extern crate nphysics3d;
extern crate uni_app;
extern crate unigame;

mod boxes_vee;
use boxes_vee::*;
use unigame::engine::*;
use std::cell::RefCell;
use std::rc::Rc;
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

struct Game {
    list: Vec<Handle<GameObject>>,
    counter: u32,
    engine: Engine,
}

impl Game {
    fn new(engine: Engine) -> Game {
        Game {
            list: Vec::new(),
            counter: 0,
            engine: engine,
        }
    }

    fn add_object(&mut self, rb: Handle<RigidBody<f32>>) -> Handle<GameObject> {
        let go = { self.engine.new_gameobject() };

        {
            let db = self.engine.asset_system();
            let mut go_mut = go.borrow_mut();
            go_mut.transform = *rb.borrow().position();

            let texture = match self.counter % 5 {
                0 => db.new_texture("tex_a.png"),
                1 => db.new_texture("tex_r.png"),
                _ => db.new_texture("tex_b.png"),
            };

            self.counter += 1;

            go_mut.add_component(self.get(rb.borrow().shape().as_ref()).unwrap());
            go_mut.add_component(Material::new_component(db.new_program("default"), texture));
            go_mut.add_component(Component::new(PhysicObject(rb)));
        }

        self.list.push(go.clone());
        go
    }

    pub fn get(&self, shape: &Shape3<f32>) -> Option<Arc<Component>> {
        let db = self.engine.asset_system();

        if let Some(_) = shape.as_shape::<Cuboid3<f32>>() {
            return Some(db.new_mesh("cube"));
        } else if let Some(_) = shape.as_shape::<Plane3<f32>>() {
            return Some(db.new_mesh("plane"));
        }

        return None;
    }
}

impl Deref for Game {
    type Target = Vec<Handle<GameObject>>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

pub fn main() {
    let size = (800, 600);
    let config = AppConfig::new("Test", size);
    let app = App::new(config);
    {
        let mut engine = Engine::new(&app, size);
        let mut scene = Scene::new();

        engine.main_camera = Some(Camera::new());

        let mut game = Game::new(engine);

        for rb in scene.world.rigid_bodies() {
            game.add_object(rb.clone());
        }

        let mut fps = FPS::new();

        app.run(move |app: &mut App| {
            game.engine.begin();
            fps.step();
            scene.step();

            imgui::pivot((0.0, 0.0));
            imgui::label(
                Metric::Native(0.0, 0.0) + Metric::Pixel(8.0, 8.0),
                &format!("fps: {} nobj: {}", fps.fps, game.len()),
            );

            imgui::pivot((1.0, 1.0));
            imgui::label(
                Metric::Native(1.0, 1.0) - Metric::Pixel(8.0, 8.0),
                "Click on canvas to drop new box.",
            );

            // Handle Events
            {
                for evt in app.events.borrow().iter() {
                    match evt {
                        &AppEvent::Click => {
                            game.add_object(scene.add_box());
                        }
                    }
                }
            }

            // Update Camera
            {
                let cam = game.engine.main_camera.as_mut().unwrap();
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

                for go in game.iter() {
                    let tran = get_pb_tran(&go.borrow());
                    go.borrow_mut().transform = tran;
                }
            }

            // Render
            game.engine.render();

            // End
            game.engine.end();
        });
    }
}
