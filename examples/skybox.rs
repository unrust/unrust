extern crate unrust;

use unrust::world::{Actor, World, WorldBuilder};
use unrust::engine::{Directional, GameObject, Light, Material, Mesh, RenderQueue};
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
            eye: Vector3::new(-3.0, 0.0, -3.0),
            last_event: None,
        })
    }

    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        // add direction light to scene.
        {
            let go = world.new_game_object();
            go.borrow_mut()
                .add_component(Light::new(Directional::default()));
        }

        // Added a SkyBox in the scene
        {
            let go = world.new_game_object();
            go.borrow_mut().add_component(SkyBox::new());
        }
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        // Handle Events
        {
            let up = Vector3::y();
            let left = up.cross(&self.eye).normalize();

            let mut reset = false;

            for evt in world.events().iter() {
                self.last_event = Some(evt.clone());
                match evt {
                    &AppEvent::KeyDown(ref key) => {
                        match key.code.as_str() {
                            "KeyA" => self.eye = Rotation3::new(up * 0.02) * self.eye,
                            "KeyD" => self.eye = Rotation3::new(up * -0.02) * self.eye,
                            "KeyW" => self.eye = Rotation3::new(left * 0.02) * self.eye,
                            "KeyS" => self.eye = Rotation3::new(left * -0.02) * self.eye,
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
            let mut cam = world.engine().main_camera.as_ref().unwrap().borrow_mut();

            cam.lookat(
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
    }
}

pub struct SkyBox {}

impl Actor for SkyBox {
    fn new() -> Box<Actor> {
        Box::new(SkyBox {})
    }

    fn start(&mut self, go: &mut GameObject, world: &mut World) {
        let db = &mut world.asset_system();

        let mut material = Material::new(db.new_program("skybox"));
        material.set("uSkybox", db.new_texture("skybox/sky_cubemap.png"));
        material.render_queue = RenderQueue::Transparent;

        let mut mesh = Mesh::new();
        mesh.add_surface(db.new_mesh_buffer("skybox"), material);
        go.add_component(mesh);
    }
}

pub fn main() {
    let mut world = WorldBuilder::new("Skybox demo")
        .with_size((640, 480))
        .with_stats(true)
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());
    drop(scene);

    world.event_loop();
}
