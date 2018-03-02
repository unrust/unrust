extern crate unrust;

use unrust::world::{Actor, Handle, World, WorldBuilder};
use unrust::engine::{Directional, GameObject, Light, Material, Mesh};
use unrust::world::events::*;
use unrust::math::*;

// GUI
use unrust::imgui;

use std::rc::Rc;

pub struct MainScene {
    cube: Handle<GameObject>,
    eye: Vector3<f32>,
}

// Actor is a trait object which would act like an component
// (Because Box<Actor> implemented ComponentBased)
impl Actor for MainScene {
    fn new() -> Box<Actor> {
        Box::new(MainScene {
            cube: Default::default(),
            eye: Vector3::new(-3.0, 3.0, -3.0),
        })
    }

    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        // add direction light to scene.
        {
            let go = world.new_game_object();

            go.borrow_mut().add_component(Light::new(Directional {
                direction: Vector3::new(0.5, -1.0, 1.0).normalize(),
                ambient: Vector3::new(0.2, 0.2, 0.2),
                diffuse: Vector3::new(0.5, 0.5, 0.5),
                specular: Vector3::new(1.0, 1.0, 1.0),
            }));
        }

        // Added a cube in the scene
        {
            let go = world.new_game_object();

            let db = &mut world.asset_system();

            let mut material = Material::new(db.new_program("phong"));
            material.set("uMaterial.diffuse", db.new_texture("tex_a.png"));
            material.set("uMaterial.shininess", 32.0);

            let mut mesh = Mesh::new();
            mesh.add_surface(db.new_mesh_buffer("cube"), Rc::new(material));
            go.borrow_mut().add_component(mesh);

            self.cube = go;
        }
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        // Rotate Cube
        self.cube
            .borrow_mut()
            .transform
            .append_rotation_mut(&UnitQuaternion::new(Vector3::new(0.01, 0.02, 0.005)));

        // Handle Events
        let mut last_event = None;
        {
            let target = Vector3::new(0.0, 0.0, 0.0);
            let front = (self.eye - target).normalize();
            let up = Vector3::y();

            let mut reset = false;

            for evt in world.events().iter() {
                last_event = Some(evt.clone());
                match evt {
                    &AppEvent::KeyDown(ref key) => {
                        match key.code.as_str() {
                            "KeyA" => self.eye = Rotation3::new(up * -0.02) * self.eye,
                            "KeyD" => self.eye = Rotation3::new(up * 0.02) * self.eye,
                            "KeyW" => self.eye -= front * 2.0,
                            "KeyS" => self.eye += front * 2.0,
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
                world.root().add_component(MainScene::new());
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
            "[Esc]  : reload all (include assets)",
        );

        imgui::pivot((1.0, 0.0));
        imgui::label(
            Native(1.0, 0.0) + Pixel(-8.0, 8.0),
            &format!("last event: {:?}", last_event),
        );
    }
}

pub fn main() {
    let world = WorldBuilder::new("Basic demo")
        .with_size((640, 480))
        .with_stats(true)
        .build();

    // Add the main scene as component of the root game object
    world.root().add_component(MainScene::new());

    world.event_loop();
}
