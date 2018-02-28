#![recursion_limit = "512"]
#![feature(integer_atomics)]

/* common */
extern crate alga;
extern crate futures;
extern crate nalgebra as na;
extern crate uni_app;
extern crate unigame;

mod appfs;

use appfs::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use na::{Matrix4, Point3, Vector3};
use std::sync::{Arc, Weak};

use unigame::engine::*;
use uni_app::{App, AppConfig, AppEvent, FPS};
use alga::linear::Transformation;

type Handle<T> = Rc<RefCell<T>>;

struct Game {
    list: Vec<Handle<GameObject>>,
    engine: AppEngine,
    point_light_coms: Vec<Weak<Component>>,
    dir_light_com: Option<Weak<Component>>,
}

fn rad(f: f32) -> f32 {
    f.to_radians()
}

impl Game {
    fn new(engine: AppEngine) -> Game {
        let mut g = Game {
            list: Vec::new(),
            engine: engine,
            point_light_coms: Vec::new(),
            dir_light_com: None,
        };

        g.setup();
        g
    }

    pub fn step(&mut self) {
        // for go in self.iter() {
        //     let mut go_mut = go.borrow_mut();
        //     go_mut
        //         .transform
        //         .append_rotation_mut(&UnitQuaternion::new(Vector3::new(0.0, 0.002, 0.0)));
        // }
    }

    pub fn reset(&mut self) {
        self.list.clear();
        self.engine.asset_system_mut().reset();
        self.point_light_coms.clear();

        self.setup();
    }

    pub fn setup(&mut self) {
        // add direction light to scene.
        let dir_light_com = {
            let go = self.engine.new_gameobject();
            // Make sure it is store some where, else it will gc
            self.push(go.clone());

            let m = Matrix4::from_euler_angles(rad(30.0), rad(50.0), 0.0);
            let light_dir = Vector3::new(0.0, 0.0, 1.0);
            let light_dir = m.transform_vector(&light_dir);

            let mut go_mut = go.borrow_mut();
            let com = go_mut.add_component(Light::new(Directional {
                direction: light_dir.normalize(),
                ambient: Vector3::new(0.212, 0.227, 0.259),
                diffuse: Vector3::new(1.0, 0.957, 0.839),
                specular: Vector3::new(1.0, 1.0, 1.0),
            }));

            com
        };

        self.dir_light_com = Some(Arc::downgrade(&dir_light_com));

        // Add 4 points light to scene
        let point_light_positions = vec![
            Vector3::new(-30.0, 30.0, -30.0),
            Vector3::new(-15.0, 300.0, -10.0),
            Vector3::new(30.0, 50.0, 30.0),
            Vector3::new(30.0, 100.0, -20.0),
        ];

        for p in point_light_positions.into_iter() {
            let go = self.engine.new_gameobject();
            // Make sure it is store some where, else it will gc
            self.push(go.clone());

            let mut go_mut = go.borrow_mut();
            let com = Light::new(Point {
                position: p,
                ambient: Vector3::new(0.05, 0.05, 0.05),
                diffuse: Vector3::new(0.2, 0.2, 0.2),
                specular: Vector3::new(0.2, 0.2, 0.2),
                constant: 1.0,
                linear: 0.022,
                quadratic: 0.0019,
            });

            self.point_light_coms
                .push(Arc::downgrade(&go_mut.add_component(com)));
        }

        let go = { self.engine.new_gameobject() };
        {
            let db = &mut self.engine.asset_system();

            let prefab_handler = {
                let go = go.clone();
                move |r: Result<Prefab, AssetError>| {
                    if let Ok(prefab) = r {
                        for c in prefab.components {
                            go.borrow_mut().add_component(c.clone());
                        }
                    }
                }
            };

            db.new_prefab("meshobj_test_model.obj", Box::new(prefab_handler));
        }
        self.list.push(go.clone());
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
    let config = AppConfig::new("Mesh (obj) demo", size);
    let app = App::new(config);

    {
        let mut game = Game::new(Engine::new(app.canvas(), size));
        game.engine.main_camera = Some(Rc::new(RefCell::new(Camera::new())));

        use imgui::Metric::*;

        let mut fps = FPS::new();
        let mut last_event = None;
        let mut eye = Vector3::new(0.0, -0.06, -3.36);
        let up = Vector3::new(0.0, 1.0, 0.0);

        app.run(move |app: &mut App| {
            game.engine.begin();
            fps.step();
            game.step();

            imgui::pivot((0.0, 0.0));
            imgui::label(
                Native(0.0, 0.0) + Pixel(8.0, 8.0),
                &format!("fps: {} nobj: {}", fps.fps, game.engine.objects.len()),
            );

            imgui::pivot((1.0, 1.0));
            imgui::label(
                Native(1.0, 1.0) - Pixel(8.0, 8.0),
                "[Esc]  : reload all (include assets)",
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
                        &AppEvent::Click(_) => {}

                        &AppEvent::KeyDown(ref key) => {
                            match key.code.as_str() {
                                "KeyA" => eye = na::Rotation3::new(up * -0.02) * eye,
                                "KeyD" => eye = na::Rotation3::new(up * 0.02) * eye,
                                "KeyW" => eye = eye - front * 0.01,
                                "KeyS" => eye = eye + front * 0.01,
                                "Escape" => game.reset(),
                                _ => (),
                            };
                        }

                        _ => (),
                    }
                }
            }

            // Update Camera
            {
                let mut cam = game.engine.main_camera.as_ref().unwrap().borrow_mut();
                cam.lookat(
                    &Point3::from_coordinates(eye),
                    &Point3::new(0.0, 0.0, 0.0),
                    &Vector3::new(0.0, 1.0, 0.0),
                );
            }

            // Update Light
            for light_com_weak in game.point_light_coms.iter() {
                if let Some(light_com) = light_com_weak.upgrade() {
                    if let Some(lr) = light_com.try_as::<Light>() {
                        let mut light = lr.borrow_mut();
                        let mut pos = light.point().unwrap().position;

                        light.point_mut().unwrap().position = na::Rotation3::new(up * 0.02) * pos;
                    }
                }
            }

            // Render
            game.engine.render(ClearOption::default());

            // End
            game.engine.end();
        });
    }
}
