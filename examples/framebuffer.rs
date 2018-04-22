extern crate unrust;

use unrust::world::{Actor, World, WorldBuilder};
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
// (Because Box<Actor> is Component)
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

        // Added a cube in the scene
        let go = world.new_game_object();
        go.borrow_mut().add_component(Cube::new());

        // Added mini screen
        let go = world.new_game_object();
        go.borrow_mut().add_component(MiniScreen::new());
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
                let scene = world.new_game_object();
                scene.borrow_mut().add_component(MainScene::new());
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
            "[WASD] : control camera\nF1 to turn framebuffer on/off.\n[Esc]  : reload all (include assets)",
        );

        imgui::pivot((1.0, 0.0));
        imgui::label(
            Native(1.0, 0.0) + Pixel(-8.0, 8.0),
            &format!("last event: {:?}", self.last_event),
        );
    }
}

pub struct MiniScreen {
    crt: bool,
    rt: Rc<RenderTexture>,
}

impl MiniScreen {
    fn new() -> Box<Actor> {
        Box::new(MiniScreen {
            crt: false,
            rt: Rc::new(RenderTexture::new(1024, 1024, TextureAttachment::Color0)),
        })
    }
}

impl Actor for MiniScreen {
    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        for evt in world.events().iter() {
            if let &AppEvent::KeyUp(ref key) = evt {
                match key.code.as_str() {
                    "F1" => self.crt = !self.crt,
                    _ => (),
                };
            }
        }

        // GUI
        use imgui::Metric::*;

        if self.crt {
            // Setup fb for camera
            let cam_borrow = world.current_camera().unwrap();
            let mut cam = cam_borrow.borrow_mut();
            cam.render_texture = Some(self.rt.clone());

            // Setup proper viewport to render to the whole texture
            cam.rect = Some(((0, 0), (1024, 1024)));

            // Render current scene by camera using given frame buffer
            world.engine_mut().render_pass(&cam, ClearOption::default());

            // Clean up stuffs in camera, as later we could render normally
            cam.render_texture = None;
            cam.rect = None;

            // render fb texture on screen
            imgui::pivot((0.0, 1.0));
            imgui::image(Native(0.0, 1.0), Pixel(300.0, 225.0), self.rt.as_texture());
        }
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

        let axis = Vector3::new(0.01, 0.02, 0.005);
        let len = axis.magnitude();
        gtran.rot = gtran.rot * Quaternion::from_axis_angle(axis.normalize(), Rad(len));

        go.transform.set_global(gtran);
    }
}

pub fn main() {
    let mut world = WorldBuilder::new("Frame Buffer demo")
        .with_size((800, 600))
        .with_stats(true)
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());
    drop(scene);

    world.event_loop();
}
