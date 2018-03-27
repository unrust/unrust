use math::{Point3, Rotation3, Vector3};
use engine::{Camera, Component, ComponentBased, GameObject};
use world::{Actor, Processor, World};
use uni_app::AppEvent;

use std::cell::RefCell;
use std::sync::Arc;

bitflags! {
    struct Movement: u32 {
        const TURN_LEFT = 1;
        const TURN_RIGHT = 1 << 2;
        const FORWARD = 1 << 3;
        const BACKWARD = 1 << 4;
        const UP = 1 << 5;
        const DOWN = 1 << 6;
        const LEFT = 1 << 7;
        const RIGHT = 1 << 8;
    }
}

pub struct FirstPersonCamera {
    pub speed: f32,
    pub eye: Vector3<f32>,
    pub eye_dir: Vector3<f32>,

    camera: Option<Arc<Component>>,

    state: Movement,
    handlers: Vec<(Movement, String, Box<Fn(&mut FirstPersonCamera)>)>,
}

impl ComponentBased for FirstPersonCamera {}

impl Processor for FirstPersonCamera {
    fn new() -> FirstPersonCamera {
        let mut m = FirstPersonCamera {
            speed: 0.1,
            state: Movement::empty(),
            handlers: Vec::new(),
            camera: None,
            eye: Vector3::new(0.0, 0.0, -3.0),
            eye_dir: Vector3::new(0.0, 0.0, 1.0).normalize(),
        };

        let up = Vector3::y();

        m.add(Movement::TURN_LEFT, "KeyA", move |s| {
            s.eye_dir = Rotation3::new(up * 0.01) * s.eye_dir;
        });
        m.add(Movement::TURN_RIGHT, "KeyD", move |s| {
            s.eye_dir = Rotation3::new(up * -0.01) * s.eye_dir
        });
        m.add(Movement::UP, "KeyE", move |s| {
            s.eye = s.eye + up * s.speed;
        });
        m.add(Movement::DOWN, "KeyC", move |s| {
            s.eye = s.eye + up * -s.speed;
        });
        m.add(Movement::FORWARD, "KeyW", move |s| {
            s.eye = s.eye + s.eye_dir * s.speed;
        });
        m.add(Movement::BACKWARD, "KeyS", move |s| {
            s.eye = s.eye + s.eye_dir * -s.speed;
        });
        m.add(Movement::LEFT, "KeyZ", move |s| {
            let right = s.eye_dir.cross(&up).normalize();
            s.eye = s.eye - right * s.speed;
        });
        m.add(Movement::RIGHT, "KeyX", move |s| {
            let right = s.eye_dir.cross(&up).normalize();
            s.eye = s.eye + right * s.speed;
        });
        m
    }
}

impl Actor for FirstPersonCamera {
    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        // add main camera to scene
        {
            let go = world.new_game_object();
            let cam = Camera::default();
            let c = go.borrow_mut().add_component(cam);

            self.camera = Some(c);
        }
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        for evt in world.events().iter() {
            self.handle_event(evt)
        }

        let cam = world.current_camera().unwrap();

        let mut handlers = Vec::new();
        handlers.append(&mut self.handlers);

        for &(ref mv, _, ref h) in handlers.iter() {
            if self.state.contains(*mv) {
                h(self);
            }
        }

        self.handlers.append(&mut handlers);

        self.update_camera();

        // Update Camera
        {
            cam.borrow_mut().lookat(
                &Point3::from_coordinates(self.eye),
                &Point3::from_coordinates(self.eye + self.eye_dir * 10.0),
                &Vector3::new(0.0, 1.0, 0.0),
            );
        }
    }
}

impl FirstPersonCamera {
    pub fn update_camera(&mut self) {
        let cam = self.camera();

        // Update Camera
        {
            cam.borrow_mut().lookat(
                &Point3::from_coordinates(self.eye),
                &Point3::from_coordinates(self.eye + self.eye_dir * 10.0),
                &Vector3::new(0.0, 1.0, 0.0),
            );
        }
    }

    fn add<F>(&mut self, mv: Movement, key: &str, f: F)
    where
        F: Fn(&mut FirstPersonCamera) + 'static,
    {
        self.handlers.push((mv, key.to_string(), Box::new(f)));
    }

    fn key_down(&mut self, input: &str) {
        for &(ref mv, ref key, _) in self.handlers.iter() {
            if input == key {
                self.state.insert(*mv)
            }
        }
    }

    fn key_up(&mut self, input: &str) {
        for &(ref mv, ref key, _) in self.handlers.iter() {
            if input == key {
                self.state.remove(*mv)
            }
        }
    }

    fn handle_event(&mut self, evt: &AppEvent) {
        match evt {
            &AppEvent::KeyUp(ref key) => {
                self.key_up(key.key.as_str());
            }

            &AppEvent::KeyDown(ref key) => {
                self.key_down(key.key.as_str());
            }

            _ => {}
        }
    }

    pub fn camera(&self) -> &RefCell<Camera> {
        self.camera.as_ref().unwrap().try_as::<Camera>().unwrap()
    }
}
