extern crate unrust;

use unrust::world::{Actor, World, WorldBuilder};
use unrust::engine::{Camera, Directional, GameObject, Light, Material, Mesh};
use unrust::world::events::*;
use unrust::math::*;
use std::f32::consts;

// GUI
use unrust::imgui;

pub struct MainScene {
    eye: Vector3<f32>,
    last_event: Option<AppEvent>,
}

// Actor is a trait object which would act like an component
// (Because Box<Actor> implemented ComponentBased)
impl MainScene {
    fn new() -> Box<Actor> {
        Box::new(MainScene {
            eye: Vector3::new(13.0, 26.0, -34.0),
            last_event: None,
        })
    }
}

impl Actor for MainScene {
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

        // Added an cemter cube in the scene
        let cube = world.new_game_object();
        cube.borrow_mut().add_component(Cube::new());
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        // Handle Events
        {
            let target = Vector3::new(0.0, 0.0, 0.0);
            let front = (self.eye - target).normalize();
            let up = Vector3::y();

            let mut reset = false;

            for evt in world.events().iter() {
                self.last_event = Some(evt.clone());
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

pub struct Cube {
    level: u32,
    radius: f32,
}

impl Cube {
    fn new() -> Box<Actor> {
        Box::new(Cube {
            level: 0,
            radius: 10.0,
        })
    }
}

impl Actor for Cube {
    fn start(&mut self, go: &mut GameObject, world: &mut World) {
        {
            let db = &mut world.asset_system();

            let material = Material::new(db.new_program("phong"));
            let s: &str = match self.level % 3 {
                0 => "tex_a.png",
                1 => "tex_b.png",
                _ => "tex_r.png",
            };

            material.set("uMaterial.diffuse", db.new_texture(s));
            material.set("uMaterial.shininess", 32.0);

            let mut mesh = Mesh::new();
            mesh.add_surface(db.new_mesh_buffer("cube"), material);
            go.add_component(mesh);
        }

        if self.level < 2 {
            for i in 0..5 {
                let cube = world.new_game_object();
                let mut cube_mut = cube.borrow_mut();

                cube_mut.add_component(Cube::new_actor(Cube {
                    level: self.level + 1,
                    radius: self.radius * 0.5,
                }));
                go.add_child(&cube_mut);

                let r = self.radius;
                let rad = ((i as f32) / 5.0) * 2.0 * consts::PI;

                let mut gtran = cube_mut.transform.local();
                gtran.translation.vector = Vector3::new(rad.sin() * r, rad.cos() * r, 0.0);
                cube_mut.transform.set_local(gtran);
            }
        }
    }

    fn update(&mut self, go: &mut GameObject, _world: &mut World) {
        let mut ltran = go.transform.local();
        ltran.append_rotation_wrt_center_mut(&UnitQuaternion::new(Vector3::new(0.01, 0.0, 0.0)));
        go.transform.set_local(ltran);
    }
}

pub fn main() {
    let mut world = WorldBuilder::new("Scene Nodes demo")
        .with_size((640, 480))
        .with_stats(true)
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());
    drop(scene);

    world.event_loop();
}
