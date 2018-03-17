extern crate unrust;
#[macro_use]
extern crate bitflags;

use unrust::world::{Actor, Handle, Processor, World, WorldBuilder};
use unrust::engine::{AssetError, Camera, ComponentBased, Directional, GameObject, Light, Material,
                     Point, Prefab};
use unrust::world::events::*;
use unrust::math::*;
use unrust::actors::SkyBox;

// GUI
use unrust::imgui;

use std::rc::Rc;

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

struct FirstPersonControl {
    eye: Vector3<f32>,
    eye_dir: Vector3<f32>,
    state: Movement,
    handlers: Vec<(Movement, String, Box<Fn(&mut FirstPersonControl)>)>,
}

impl FirstPersonControl {
    fn add<F>(&mut self, mv: Movement, key: &str, f: F)
    where
        F: Fn(&mut FirstPersonControl) + 'static,
    {
        self.handlers.push((mv, key.to_string(), Box::new(f)));
    }

    fn new() -> FirstPersonControl {
        let mut m = FirstPersonControl {
            state: Movement::empty(),
            handlers: Vec::new(),
            eye: Vector3::new(0.0, 200.06, -3.36),
            eye_dir: Vector3::new(-3.0, 0.0, -1.0).normalize(),
        };

        let speed = 8.0;
        let up = Vector3::y();

        m.add(Movement::TURN_LEFT, "KeyA", move |s| {
            s.eye_dir = Rotation3::new(up * 0.01) * s.eye_dir;
        });
        m.add(Movement::TURN_RIGHT, "KeyD", move |s| {
            s.eye_dir = Rotation3::new(up * -0.01) * s.eye_dir
        });
        m.add(Movement::UP, "KeyE", move |s| {
            s.eye = s.eye + up * speed;
        });
        m.add(Movement::DOWN, "KeyC", move |s| {
            s.eye = s.eye + up * -speed;
        });
        m.add(Movement::FORWARD, "KeyW", move |s| {
            s.eye = s.eye + s.eye_dir * speed;
        });
        m.add(Movement::BACKWARD, "KeyS", move |s| {
            s.eye = s.eye + s.eye_dir * -speed;
        });
        m.add(Movement::LEFT, "KeyZ", move |s| {
            let right = s.eye_dir.cross(&up).normalize();
            s.eye = s.eye - right * speed;
        });
        m.add(Movement::RIGHT, "KeyX", move |s| {
            let right = s.eye_dir.cross(&up).normalize();
            s.eye = s.eye + right * speed;
        });
        m
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

    fn update_camera(&mut self, cam: &mut Camera) {
        let mut handlers = Vec::new();
        handlers.append(&mut self.handlers);

        for &(ref mv, _, ref h) in handlers.iter() {
            if self.state.contains(*mv) {
                h(self);
            }
        }

        self.handlers.append(&mut handlers);

        // Update Camera
        {
            cam.lookat(
                &Point3::from_coordinates(self.eye),
                &Point3::from_coordinates(self.eye + self.eye_dir * 10.0),
                &Vector3::new(0.0, 1.0, 0.0),
            );
        }
    }
}

pub struct MainScene {
    dir_light: Handle<GameObject>,
    point_light: Handle<GameObject>,

    animate_light: bool,
    camera_control: FirstPersonControl,

    last_event: Option<AppEvent>,
}

pub struct MaterialFilter {
    force_no_normal_map: bool,
}

impl ComponentBased for MaterialFilter {}
impl Actor for MaterialFilter {}

impl Processor for MaterialFilter {
    fn new() -> MaterialFilter {
        MaterialFilter {
            force_no_normal_map: false,
        }
    }

    fn apply_materials(&self, materials: &Vec<Rc<Material>>) {
        for m in materials.iter() {
            m.set("uNoNormalMap", self.force_no_normal_map);
        }
    }

    fn watch_material() -> bool
    where
        Self: Sized,
    {
        return true;
    }
}

// Actor is a trait object which would act like an component
// (Because Box<Actor> implemented ComponentBased)
impl MainScene {
    fn new() -> Box<Actor> {
        Box::new(MainScene {
            last_event: None,
            animate_light: true,
            dir_light: GameObject::empty(),
            point_light: GameObject::empty(),
            camera_control: FirstPersonControl::new(),
        })
    }
}

impl Actor for MainScene {
    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        // add main camera to scene
        {
            let go = world.new_game_object();
            let mut cam = Camera::default();
            cam.znear = 1.0;
            cam.zfar = 10000.0;
            go.borrow_mut().add_component(cam);
        }

        // add direction light to scene.
        {
            let go = world.new_game_object();
            go.borrow_mut()
                .add_component(Light::new(Directional::default()));
            self.dir_light = go;
        }

        {
            let go = world.new_game_object();
            go.borrow_mut().add_component(Light::new(Point::default()));
            self.point_light = go;
        }

        // Added the obj display
        {
            let go = world.new_game_object();
            go.borrow_mut().add_component(WaveObjActor::new());
        }
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        // Handle Events
        let normap_map_enabled;

        {
            let mut reset = false;
            let mut toggle_normal_map = false;

            for evt in world.events().iter() {
                self.last_event = Some(evt.clone());
                match evt {
                    &AppEvent::KeyUp(ref key) => {
                        self.camera_control.key_up(key.code.as_str());
                        match key.code.as_str() {
                            "KeyU" => toggle_normal_map = true,
                            "Space" => self.animate_light = !self.animate_light,
                            "Escape" => reset = true,

                            _ => (),
                        }
                    }

                    &AppEvent::KeyDown(ref key) => {
                        self.camera_control.key_down(key.code.as_str());
                    }

                    _ => (),
                }
            }

            if reset {
                world.reset();
                // Because reset will remove all objects in the world,
                // included this Actor itself
                // so will need to add it back.
                let scene = world.new_game_object();
                scene.borrow_mut().add_component(MainScene::new());
                return;
            }

            if toggle_normal_map {
                let mf = world.find_component::<MaterialFilter>().unwrap();
                let b = mf.borrow().force_no_normal_map;
                mf.borrow_mut().force_no_normal_map = !b;
            }

            normap_map_enabled = {
                let mf = world.find_component::<MaterialFilter>().unwrap();
                let mf_borrow = mf.borrow();
                !mf_borrow.force_no_normal_map
            };
        }

        // Update Movement
        {
            let cam = world.current_camera().unwrap();
            self.camera_control.update_camera(&mut cam.borrow_mut());
        }

        // Update Direction light
        if self.animate_light {
            let dir_light_bor = self.dir_light.borrow_mut();
            let (mut light, _) = dir_light_bor.find_component_mut::<Light>().unwrap();

            let dir = light.directional().unwrap().direction;
            let mut t = Isometry3::identity();
            t.append_rotation_mut(&UnitQuaternion::new(Vector3::new(0.0, 0.01, 0.0)));
            let transformed = t * dir;

            light.directional_mut().unwrap().direction = transformed;
        }

        // Update Point Light
        {
            let point_light_bor = self.point_light.borrow_mut();
            let (mut light, _) = point_light_bor.find_component_mut::<Light>().unwrap();
            light.point_mut().unwrap().linear = 0.0007;
            light.point_mut().unwrap().quadratic = 0.00002;
            light.point_mut().unwrap().position = self.camera_control.eye;
        }

        // GUI
        use imgui::Metric::*;
        use imgui::TextAlign::*;

        imgui::pivot((1.0, 1.0));
        imgui::text_align(Left);
        imgui::label(
            Native(1.0, 1.0) - Pixel(8.0, 8.0),
            "[WASD ZXEC] : control camera\n[Space] : Toggle light animation\n[U] : Toggle normal map\n[Esc] : reload all (include assets)",
        );

        imgui::pivot((1.0, 0.0));
        imgui::text_align(Right);
        imgui::label(
            Native(1.0, 0.0) + Pixel(-8.0, 8.0),
            &format!(
                "last event: {:?}\nnormal_map = {:?}\nlight animation={:?}",
                self.last_event, normap_map_enabled, self.animate_light
            ),
        );

        imgui::pivot((0.0, 1.0));
        imgui::text_align(Left);
        imgui::label(Native(0.0, 1.0) + Pixel(8.0, -8.0), "Sponza Demo");
    }
}

pub struct WaveObjActor {}

impl WaveObjActor {
    fn new() -> Box<Actor> {
        Box::new(WaveObjActor {})
    }
}

impl Actor for WaveObjActor {
    fn start_rc(&mut self, go: Handle<GameObject>, world: &mut World) {
        let db = &mut world.asset_system();

        let prefab_handler = {
            let go = go.clone();
            move |r: Result<Prefab, AssetError>| match r {
                Ok(prefab) => for c in prefab.components {
                    go.borrow_mut().add_component(c.clone());
                },
                Err(err) => {
                    panic!(format!("Cannot load prefab, reason:{:?}", err));
                }
            }
        };

        db.new_prefab("sponza/sponza.obj", Box::new(prefab_handler));
    }
}

pub fn main() {
    let mut world = WorldBuilder::new("sponza demo")
        .with_size((800, 600))
        .with_stats(true)
        .with_processor::<MaterialFilter>()
        .with_processor::<SkyBox>()
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());
    drop(scene);

    world.event_loop();
}
