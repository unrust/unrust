#![recursion_limit = "512"]
#![feature(integer_atomics)]

/* common */
extern crate futures;
extern crate nalgebra as na;
extern crate ncollide;
extern crate nphysics3d;
extern crate uni_app;
extern crate unigame;

mod boxes_vee;
mod appfs;

use boxes_vee::*;
use appfs::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use na::Isometry3;
use std::collections::HashMap;

use nphysics3d::object::RigidBody;
use na::{Point3, Vector3};
use ncollide::shape::{Cuboid3, Plane3, Shape3};

use unigame::engine::*;
use uni_app::{App, AppConfig, AppEvent, FPS};

type Handle<T> = Rc<RefCell<T>>;

// Physic Object Component
struct PhysicObject(Handle<RigidBody<f32>>);
impl PhysicObject {
    fn phy_transform(&self) -> Isometry3<f32> {
        *self.0.borrow().position()
    }
}
impl ComponentBased for PhysicObject {}

struct Game {
    list: Vec<Handle<GameObject>>,
    counter: u32,
    engine: AppEngine,
}

impl Game {
    fn new(engine: AppEngine) -> Game {
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

            let mut textures = HashMap::new();
            textures.insert(
                "uMaterial.diffuse".to_string(),
                MaterialParam::Texture(texture),
            );
            textures.insert(
                "uMaterial.shininess".to_string(),
                MaterialParam::Float(32.0),
            );

            // temp set the material shiness here

            go_mut.add_component(self.get(rb.borrow().shape().as_ref()).unwrap());
            go_mut.add_component(PhysicObject(rb));
            go_mut.add_component(Material::new(db.new_program("phong"), textures));
        }

        self.list.push(go.clone());
        go
    }

    pub fn get(&self, shape: &Shape3<f32>) -> Option<Mesh> {
        let db = self.engine.asset_system();

        if let Some(_) = shape.as_shape::<Cuboid3<f32>>() {
            Some(Mesh::new(db.new_mesh_buffer("cube")))
        } else if let Some(_) = shape.as_shape::<Plane3<f32>>() {
            Some(Mesh::new(db.new_mesh_buffer("plane")))
        } else {
            None
        }
    }
}

impl Deref for Game {
    type Target = Vec<Handle<GameObject>>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for Game {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

pub fn main() {
    let size = (800, 600);
    let config = AppConfig::new("Test", size);
    let app = App::new(config);
    {
        let mut scene = Scene::new();

        let mut game = Game::new(Engine::new(app.canvas(), size));
        game.engine.main_camera = Some(Camera::new());

        for rb in scene.world.rigid_bodies() {
            game.add_object(rb.clone());
        }

        // add direction light to scene.
        let _dir_light_com = {
            let go = game.engine.new_gameobject();
            // Make sure it is store some where, else it will gc
            game.push(go.clone());

            let mut go_mut = go.borrow_mut();
            let com = go_mut.add_component(Light::new(Directional {
                direction: Vector3::new(0.5, -1.0, 1.0).normalize(),
                ambient: Vector3::new(0.2, 0.2, 0.2),
                diffuse: Vector3::new(0.5, 0.5, 0.5),
                specular: Vector3::new(1.0, 1.0, 1.0),
            }));

            com
        };

        // Add 4 points light to scene
        let point_light_positions = vec![
            Vector3::new(-30.0, 30.0, -30.0),
            Vector3::new(-15.0, 300.0, -10.0),
            Vector3::new(30.0, 50.0, 30.0),
            Vector3::new(30.0, 100.0, -20.0),
        ];

        let mut point_light_coms = vec![];
        for p in point_light_positions.into_iter() {
            let go = game.engine.new_gameobject();
            // Make sure it is store some where, else it will gc
            game.push(go.clone());

            let mut go_mut = go.borrow_mut();

            point_light_coms.push(go_mut.add_component(Light::new(Point {
                position: p,
                ambient: Vector3::new(0.05, 0.05, 0.05),
                diffuse: Vector3::new(0.8, 0.8, 0.8),
                specular: Vector3::new(1.0, 1.0, 1.0),
                constant: 1.0,
                linear: 0.022,
                quadratic: 0.0019,
            })));
        }

        use imgui::Metric::*;

        let mut fps = FPS::new();
        let mut last_event = None;
        let mut eye = Vector3::new(-30.0, 30.0, -30.0);
        let up = Vector3::new(0.0, 1.0, 0.0);

        app.run(move |app: &mut App| {
            game.engine.begin();
            fps.step();
            scene.step();

            imgui::pivot((0.0, 0.0));
            imgui::label(
                Native(0.0, 0.0) + Pixel(8.0, 8.0),
                &format!("fps: {} nobj: {}", fps.fps, game.engine.objects.len()),
            );

            imgui::pivot((1.0, 1.0));
            imgui::label(
                Native(1.0, 1.0) - Pixel(8.0, 8.0),
                "Click on canvas to drop new box. Use WASD to control camera.",
            );

            imgui::pivot((1.0, 0.0));
            imgui::label(
                Native(1.0, 0.0) + Pixel(-8.0, 8.0),
                &format!("last event: {:?}", last_event),
            );

            // Handle Events
            {
                let target = Vector3::new(0.0, 0.0, 0.0);
                let front = (eye - target).normalize();

                let events = app.events.borrow();
                for evt in events.iter() {
                    last_event = Some(evt.clone());
                    match evt {
                        &AppEvent::Click(_) => {
                            game.add_object(scene.add_box());
                        }

                        &AppEvent::KeyDown(ref key) => {
                            match key.code.as_str() {
                                "KeyA" => eye = na::Rotation3::new(up * -0.02) * eye,
                                "KeyD" => eye = na::Rotation3::new(up * 0.02) * eye,
                                "KeyW" => eye = eye - front * 2.0,
                                "KeyS" => eye = eye + front * 2.0,
                                _ => (),
                            };
                        }

                        _ => (),
                    }
                }
            }

            // Update Camera
            {
                let cam = game.engine.main_camera.as_mut().unwrap();
                cam.lookat(
                    &Point3::from_coordinates(eye),
                    &Point3::new(0.0, 0.0, 0.0),
                    &Vector3::new(0.0, 1.0, 0.0),
                );
            }

            // Update Light
            for light_com in point_light_coms.iter_mut() {
                if let Some(lr) = light_com.try_as::<Light>() {
                    let mut light = lr.borrow_mut();
                    let mut pos = light.point().unwrap().position;

                    light.point_mut().unwrap().position = na::Rotation3::new(up * 0.02) * pos;
                }
            }

            // Update Transforms by physic object
            {
                let get_pb_tran = |o: &GameObject| {
                    if let Some((rb, _)) = o.find_component::<PhysicObject>() {
                        Some(rb.borrow().phy_transform())
                    } else {
                        None
                    }
                };

                for go in game.iter() {
                    let mut go_mut = go.borrow_mut();
                    if let Some(tran) = get_pb_tran(&go_mut) {
                        go_mut.transform = tran;
                    }
                }
            }

            // Render
            game.engine.render();

            // End
            game.engine.end();
        });
    }
}
