extern crate unrust;

use unrust::world::{Actor, Handle, World, WorldBuilder};
use unrust::engine::{AssetError, Camera, Directional, GameObject, Light, Prefab};
use unrust::world::events::*;
use unrust::math::*;

// GUI
use unrust::imgui;

pub struct MainScene {
    eye: Vector3<f32>,
    last_event: Option<AppEvent>,
}

// Actor is a trait object which would act like an component
// (Because Box<Actor> implemented ComponentBased)
impl Actor for MainScene {
    fn new() -> Box<Actor> {
        Box::new(MainScene {
            eye: Vector3::new(0.0, -0.06, -3.36),
            last_event: None,
        })
    }

    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        // add main camera to scene
        {
            let go = world.new_game_object();
            go.borrow_mut().add_component(Camera::default());
        }

        // add direction light to scene.
        {
            let go = world.new_game_object();
            go.borrow_mut()
                .add_component(Light::new(Directional::default()));
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

            let mut reset = false;

            for evt in world.events().iter() {
                self.last_event = Some(evt.clone());
                match evt {
                    &AppEvent::KeyDown(ref key) => {
                        match key.code.as_str() {
                            "KeyA" => self.eye = Rotation3::new(up * -0.02) * self.eye,
                            "KeyD" => self.eye = Rotation3::new(up * 0.02) * self.eye,
                            "KeyW" => self.eye *= 0.98,
                            "KeyS" => self.eye *= 1.02,
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
            }
        }

        // Update Camera
        {
            let cam = world.current_camera().unwrap();

            cam.borrow_mut().lookat(
                &Point3::from_coordinates(self.eye),
                &Point3::new(0.0, 0.0, 0.0),
                &Vector3::new(0.0, 1.0, 0.0),
            );
        }

        // GUI
        use imgui::Metric::*;

        imgui::pivot((1.0, 1.0));
        imgui::label(
            Native(1.0, 1.0) - Pixel(8.0, 8.0),
            "[WASD] : control camera\n[Esc]  : reload all (include assets)",
        );

        imgui::pivot((1.0, 0.0));
        imgui::label(
            Native(1.0, 0.0) + Pixel(-8.0, 8.0),
            &format!("last event: {:?}", self.last_event),
        );

        imgui::pivot((0.0, 1.0));
        imgui::label(
            Native(0.0, 1.0) + Pixel(8.0, -8.0),
            "Vending Machine by Don Carson\nhttps://poly.google.com/view/0CX6wj64Swu",
        );
    }
}

pub struct WaveObjActor {}

impl Actor for WaveObjActor {
    fn new() -> Box<Actor> {
        Box::new(WaveObjActor {})
    }

    fn start_rc(&mut self, go: Handle<GameObject>, world: &mut World) {
        let db = &mut world.asset_system();

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
}

pub fn main() {
    let mut world = WorldBuilder::new("obj prefab demo")
        .with_size((800, 600))
        .with_stats(true)
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());
    drop(scene);

    world.event_loop();
}
