extern crate unrust;

use unrust::world::{Actor, Handle, World, WorldBuilder};
use unrust::engine::{AssetError, Camera, Directional, GameObject, Light, Prefab};
use unrust::world::events::*;
use unrust::math::*;

// GUI
use unrust::imgui;

pub struct MainScene {
    eye: Vector3<f32>,
    eye_dir: Vector3<f32>,

    dir_light: Handle<GameObject>,

    last_event: Option<AppEvent>,
}

// Actor is a trait object which would act like an component
// (Because Box<Actor> implemented ComponentBased)
impl MainScene {
    fn new() -> Box<Actor> {
        Box::new(MainScene {
            eye: Vector3::new(0.0, 200.06, -3.36),
            eye_dir: Vector3::new(-3.0, 0.0, -1.0).normalize(),
            last_event: None,
            dir_light: GameObject::empty(),
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

        // Added the obj display
        {
            let go = world.new_game_object();
            go.borrow_mut().add_component(WaveObjActor::new());
        }
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        // Handle Events
        {
            let up = Vector3::y();
            let speed = 10.0;

            let mut reset = false;

            for evt in world.events().iter() {
                self.last_event = Some(evt.clone());
                match evt {
                    &AppEvent::KeyDown(ref key) => {
                        match key.code.as_str() {
                            "KeyA" => self.eye_dir = Rotation3::new(up * 0.02) * self.eye_dir,
                            "KeyD" => self.eye_dir = Rotation3::new(up * -0.02) * self.eye_dir,
                            "KeyE" => self.eye = self.eye + up * speed,
                            "KeyC" => self.eye = self.eye + up * -speed,

                            "KeyW" => self.eye = self.eye + self.eye_dir * speed,
                            "KeyS" => self.eye = self.eye + self.eye_dir * -speed,
                            "Escape" => reset = true,
                            _ => (),
                        };
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
        }

        // Update Camera
        {
            let cam = world.current_camera().unwrap();

            cam.borrow_mut().lookat(
                &Point3::from_coordinates(self.eye),
                &Point3::from_coordinates(self.eye + self.eye_dir * 10.0),
                &Vector3::new(0.0, 1.0, 0.0),
            );
        }

        // Update light
        {
            let dir_light_bor = self.dir_light.borrow_mut();
            let (mut light, _) = dir_light_bor.find_component_mut::<Light>().unwrap();

            let dir = light.directional().unwrap().direction;
            let mut t = Isometry3::identity();
            t.append_rotation_mut(&UnitQuaternion::new(Vector3::new(0.0, 0.01, 0.0)));
            let transformed = t * dir;

            light.directional_mut().unwrap().direction = transformed;
        }

        // GUI
        use imgui::Metric::*;

        imgui::pivot((1.0, 1.0));
        imgui::label(
            Native(1.0, 1.0) - Pixel(8.0, 8.0),
            "[WASD EC] : control camera\n[Esc]  : reload all (include assets)",
        );

        imgui::pivot((1.0, 0.0));
        imgui::label(
            Native(1.0, 0.0) + Pixel(-8.0, 8.0),
            &format!("last event: {:?}", self.last_event),
        );

        imgui::pivot((0.0, 1.0));
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
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());
    drop(scene);

    world.event_loop();
}
