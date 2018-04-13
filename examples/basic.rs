extern crate uni_pad;
extern crate unrust;

use uni_pad::{gamepad_axis, gamepad_button};
use unrust::actors::FirstPersonCamera;
use unrust::engine::{Directional, GameObject, Light, Material, Mesh};
use unrust::math::*;
use unrust::world::events::*;
use unrust::world::{Actor, World, WorldBuilder};

// GUI
use unrust::imgui;

pub struct MainScene {
    last_event: Option<AppEvent>,
}

// Actor is a trait object which would act like an component
// (Because Box<Actor> implemented ComponentBased)
impl MainScene {
    fn new() -> Box<Actor> {
        Box::new(MainScene { last_event: None })
    }
}

impl Actor for MainScene {
    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        // add direction light to scene.
        {
            let go = world.new_game_object();
            go.borrow_mut()
                .add_component(Light::new(Directional::default()));
        }

        // Added a cube in the scene
        {
            let go = world.new_game_object();
            go.borrow_mut().add_component(Cube::new());
        }

        // Setup camera
        {
            let fpc = world.find_component::<FirstPersonCamera>().unwrap();
            fpc.borrow_mut().eye = Vector3::new(0.0, 0.0, -9.0);
            fpc.borrow_mut().update_camera();
        }
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        // Handle Events
        {
            let mut reset = false;

            for evt in world.events().iter() {
                self.last_event = Some(evt.clone());
                match evt {
                    &AppEvent::KeyDown(ref key) => {
                        match key.code.as_str() {
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

        // GUI
        use imgui::Metric::*;

        imgui::pivot((1.0, 1.0));
        imgui::label(
            Native(1.0, 1.0) - Pixel(8.0, 8.0),
            &format!("[WASD ZXEC] : control camera\n[Esc] : reload all (include assets)\ngamepad: {:?} buttons {} {} {} {}",
                gamepad_axis(0),
                gamepad_button(0, 0),
                gamepad_button(0, 1),
                gamepad_button(0, 2),
                gamepad_button(0, 3))
        );

        imgui::pivot((1.0, 0.0));
        imgui::label(
            Native(1.0, 0.0) + Pixel(-8.0, 8.0),
            &format!("last event: {:?}", self.last_event),
        );
    }
}

pub struct Cube {}

impl Cube {
    fn new() -> Box<Actor> {
        Box::new(Cube {})
    }
}

impl Actor for Cube {
    fn start(&mut self, go: &mut GameObject, world: &mut World) {
        let db = &mut world.asset_system();

        let material = Material::new(db.new_program("phong"));
        material.set("uMaterial.diffuse", db.new_texture("tex_r.dds"));
        material.set("uMaterial.shininess", 32.0);

        let mut mesh = Mesh::new();
        mesh.add_surface(db.new_mesh_buffer("cube"), material);
        go.add_component(mesh);
    }

    fn update(&mut self, go: &mut GameObject, _world: &mut World) {
        let mut gtran = go.transform.global();
        let axis = Vector3::new(0.01, 0.02, 0.005);
        let len = axis.magnitude();

        gtran.rot = gtran.rot * Quaternion::from_axis_angle(axis.normalize(), Rad(len));
        go.transform.set_global(gtran);
    }
}

pub fn main() {
    let mut world = WorldBuilder::new("Basic demo")
        .with_size((640, 480))
        .with_stats(true)
        .with_processor::<FirstPersonCamera>()  // Use first person camera
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());
    drop(scene);

    world.event_loop();
}
