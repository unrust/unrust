extern crate unrust;

use unrust::world::{Actor, Handle, World, WorldBuilder};
use unrust::engine::{Camera, ClearOption, Directional, GameObject, Light, Material, Mesh,
                     RenderTexture, TextureAttachment};
use unrust::world::events::*;
use unrust::math::*;

// GUI
use unrust::imgui;

use std::rc::Rc;

pub struct MainScene {
    eye: Vector3<f32>,
    last_event: Option<AppEvent>,
}

// Actor is a trait object which would act like an component
// (Because Box<Actor> implemented ComponentBased)
impl MainScene {
    fn new() -> Box<Actor> {
        Box::new(MainScene {
            eye: Vector3::new(-3.0, 3.0, -3.0),
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
        let go = world.new_game_object();
        go.borrow_mut()
            .add_component(Light::new(Directional::default()));

        // Added Crt
        let go = world.new_game_object();
        go.borrow_mut().add_component(Crt::new());
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

pub struct Crt {
    rt: Rc<RenderTexture>,
    cube: Handle<GameObject>,
}

impl Crt {
    fn new() -> Box<Actor> {
        Box::new(Crt {
            rt: Rc::new(RenderTexture::new(1024, 1024, TextureAttachment::Color0)),
            cube: GameObject::empty(),
        })
    }
}

impl Actor for Crt {
    fn start(&mut self, go: &mut GameObject, world: &mut World) {
        // add main camera to scene
        {
            let go = world.new_game_object();
            go.borrow_mut().add_component(Camera::default());
        }

        {
            let db = &mut world.asset_system();

            let material = Material::new(db.new_program("crt"));
            material.set("uDiffuse", self.rt.as_texture());

            let mut mesh = Mesh::new();
            mesh.add_surface(db.new_mesh_buffer("screen_quad"), material);
            go.add_component(mesh);
        }

        // Added a cube in the scene
        self.cube = world.new_game_object();
        self.cube.borrow_mut().add_component(Cube::new());
    }

    fn update(&mut self, go: &mut GameObject, world: &mut World) {
        // Setup fb for camera
        let cam_borrow = world.current_camera().unwrap();
        let mut cam = cam_borrow.borrow_mut();

        cam.render_texture = Some(self.rt.clone());

        // Setup proper viewport to render to the whole texture
        cam.rect = Some(((0, 0), (1024, 1024)));

        // show only cube
        // TODO it is a little bit hacky, we should support a PostProcessing Component
        self.cube.borrow_mut().active = true;
        go.active = false;

        // Render current scene by camera using given frame buffer
        world.engine().render_pass(&cam, ClearOption::default());

        // show only this crt
        self.cube.borrow_mut().active = false;
        go.active = true;

        // Clean up stuffs in camera, as later we could render normally
        cam.render_texture = None;
        cam.rect = None;
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
        material.set("uMaterial.diffuse", db.new_texture("tex_a.png"));
        material.set("uMaterial.shininess", 32.0);

        let mut mesh = Mesh::new();
        mesh.add_surface(db.new_mesh_buffer("cube"), material);
        go.add_component(mesh);
    }

    fn update(&mut self, go: &mut GameObject, _world: &mut World) {
        let mut gtran = go.transform.global();
        gtran.append_rotation_mut(&UnitQuaternion::new(Vector3::new(0.01, 0.02, 0.005)));
        go.transform.set_global(gtran);
    }
}

pub fn main() {
    let mut world = WorldBuilder::new("Post Processing demo")
        .with_size((800, 600))
        .with_stats(true)
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());
    drop(scene);

    world.event_loop();
}
