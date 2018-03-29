extern crate unrust;

use unrust::world::{Actor, World, WorldBuilder};
use unrust::engine::{Camera, Directional, GameObject, Light, Material, Mesh};
use unrust::world::events::*;
use unrust::math::*;
use unrust::actors::ShadowPass;

// GUI
use unrust::imgui;

pub struct MainScene {
    eye: Vector3<f32>,
    last_event: Option<AppEvent>,
}

impl MainScene {
    fn new() -> Box<Actor> {
        Box::new(MainScene {
            eye: Vector3::new(12.0, 12.0, -12.0),
            last_event: None,
        })
    }

    fn build(world: &mut World) {
        let scene = world.new_game_object();
        scene.borrow_mut().add_component(MainScene::new());
    }
}

// Actor is a trait object which would act like an component
// (Because Box<Actor> implemented ComponentBased)
impl Actor for MainScene {
    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        // add main camera to scene
        {
            let go = world.new_game_object();
            let mut cam = Camera::default();
            cam.znear = 0.3;
            cam.zfar = 1000.0;
            go.borrow_mut().add_component(cam);
        }

        // add direction light to scene.
        let light = world.new_game_object();
        light
            .borrow_mut()
            .add_component(Light::new(Directional::default()));

        // Added a rotating cube in the scene
        {
            let cube = world.new_game_object();
            cube.borrow_mut()
                .add_component(Cube::new_actor(Cube { rotating: true }));
            let mut gtran = cube.borrow_mut().transform.global();
            gtran.disp = Vector3::new(0.0, 3.0, 0.0);
            cube.borrow_mut().transform.set_global(gtran);
        }

        // Added a stationary cube in the scene
        {
            let cube = world.new_game_object();
            cube.borrow_mut()
                .add_component(Cube::new_actor(Cube { rotating: false }));
            let mut gtran = cube.borrow_mut().transform.global();
            gtran.disp = Vector3::new(5.0, 1.0, 0.0);

            cube.borrow_mut().transform.set_global(gtran);
        }

        // Added a plane in the scene
        {
            let plane = world.new_game_object();
            plane.borrow_mut().add_component(Plane::new_actor(Plane {}));
        }
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        // Handle Events
        {
            let target = Vector3::new(0.0, 0.0, 0.0);
            let front = (self.eye - target).normalize();

            let mut reset = false;

            for evt in world.events().iter() {
                self.last_event = Some(evt.clone());
                match evt {
                    &AppEvent::KeyDown(ref key) => {
                        match key.code.as_str() {
                            "KeyA" => self.eye = Quaternion::from_angle_y(Rad(-0.2)) * self.eye,
                            "KeyD" => self.eye = Quaternion::from_angle_y(Rad(0.2)) * self.eye,
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
                MainScene::build(world);
                return;
            }
        }

        // Update Camera
        {
            let cam = world.current_camera().unwrap();

            cam.borrow_mut().lookat(
                &Point3::from_vec(self.eye),
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
    rotating: bool,
}

impl Actor for Cube {
    fn start(&mut self, go: &mut GameObject, world: &mut World) {
        let db = &mut world.asset_system();
        let material = Material::new(db.new_program("unrust/phong_shadow"));
        material.set("uMaterial.diffuse", db.new_texture("tex_a.png"));
        material.set("uMaterial.shininess", 32.0);

        let mut mesh = Mesh::new();
        mesh.add_surface(db.new_mesh_buffer("cube"), material);
        go.add_component(mesh);
    }

    fn update(&mut self, go: &mut GameObject, _world: &mut World) {
        if self.rotating {
            let mut gtran = go.transform.global();
            gtran.rot = gtran.rot * Quaternion::from(Euler::new(Rad(0.01), Rad(0.02), Rad(0.005)));
            go.transform.set_global(gtran);
        }
    }
}

pub struct Plane;

impl Actor for Plane {
    fn start(&mut self, go: &mut GameObject, world: &mut World) {
        let db = &mut world.asset_system();

        let material = Material::new(db.new_program("unrust/phong_shadow"));
        material.set("uMaterial.diffuse", db.new_texture("tex_a.png"));
        material.set("uMaterial.shininess", 32.0);

        let mut mesh = Mesh::new();
        mesh.add_surface(db.new_mesh_buffer("plane"), material);
        go.add_component(mesh);
    }
}

pub fn main() {
    let mut world = WorldBuilder::new("Shadow demo")
        .with_size((800, 600))
        .with_stats(true)
        .with_processor::<ShadowPass>()
        .build();

    // Add the main scene as component of scene game object
    MainScene::build(&mut world);

    world.event_loop();
}
