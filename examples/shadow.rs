extern crate unrust;

use unrust::world::{Actor, Handle, World, WorldBuilder};
use unrust::engine::{Camera, ClearOption, Directional, GameObject, Light, Material, Mesh,
                     RenderTexture, Texture, TextureAttachment};
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
impl Actor for MainScene {
    fn new() -> Box<Actor> {
        Box::new(MainScene {
            eye: Vector3::new(12.0, 12.0, -12.0),
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
        let go = world.new_game_object();
        go.borrow_mut()
            .add_component(Light::new(Directional::default()));

        // Added Shadow
        let go = world.new_game_object();
        go.borrow_mut().add_component(Shadow::new());
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

pub struct Shadow {
    rt: Rc<RenderTexture>,
    light: Handle<GameObject>,

    shadow_material: Option<Material>,
}

fn compute_light_matrix(go: &Handle<GameObject>) -> Matrix4<f32> {
    let light_borrow = go.borrow();
    let (light, _) = light_borrow.find_component::<Light>().unwrap();
    let lightdir = light.directional().unwrap().direction;

    // build an ortho matrix for directional light
    let proj = Matrix4::new_orthographic(-20.0, 20.0, -20.0, 20.0, -20.0, 40.0);
    let light_target = Point3 { coords: -lightdir };
    let view = Matrix4::look_at_rh(&light_target, &Point3::new(0.0, 0.0, 0.0), &Vector3::y());

    return proj * view;
}

impl Actor for Shadow {
    fn new() -> Box<Actor> {
        Box::new(Shadow {
            rt: Rc::new(RenderTexture::new(1024, 1024, TextureAttachment::Depth)),
            light: GameObject::empty(),
            shadow_material: None,
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
            self.light = go;
        }

        // Added a cube in the scene
        {
            let cube = world.new_game_object();
            cube.borrow_mut().add_component(Cube::new_actor(Cube {
                shadow_map: Some(self.rt.as_texture().clone()),
                light: self.light.clone(),
                rotating: true,
            }));
            let mut gtran = cube.borrow_mut().transform.global();
            gtran.append_translation_mut(&Translation3::new(0.0, 3.0, 0.0));
            cube.borrow_mut().transform.set_global(gtran);
        }

        {
            let cube = world.new_game_object();
            cube.borrow_mut().add_component(Cube::new_actor(Cube {
                shadow_map: Some(self.rt.as_texture().clone()),
                light: self.light.clone(),
                rotating: false,
            }));
            let mut gtran = cube.borrow_mut().transform.global();
            gtran.append_translation_mut(&Translation3::new(5.0, 1.0, 0.0));
            cube.borrow_mut().transform.set_global(gtran);
        }

        // Added a plane in the scene
        let plane = world.new_game_object();
        plane.borrow_mut().add_component(Plane::new_actor(Plane {
            shadow_map: Some(self.rt.as_texture().clone()),
            light: self.light.clone(),
        }));

        let db = &mut world.asset_system();
        let shadow_mat = Material::new(db.new_program("unrust/shadow"));
        self.shadow_material = Some(shadow_mat);
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        // Setup fb for camera
        let cam_borrow = world.current_camera().unwrap();
        let mut cam = cam_borrow.borrow_mut();

        {
            let light_matrix = compute_light_matrix(&self.light);

            self.shadow_material
                .as_ref()
                .unwrap()
                .set("uShadowMatrix", light_matrix);
        }

        cam.render_texture = Some(self.rt.clone());

        // Setup proper viewport to render to the whole texture
        cam.rect = Some(((0, 0), (1024, 1024)));

        // Render current scene by camera using given frame buffer
        world.engine().render_pass_with_material(
            &cam,
            self.shadow_material.as_ref(),
            ClearOption::default(),
        );

        // Clean up stuffs in camera, as later we could render normally
        cam.render_texture = None;
        cam.rect = None;
    }
}

pub struct Cube {
    shadow_map: Option<Rc<Texture>>,
    light: Handle<GameObject>,
    rotating: bool,
}

impl Actor for Cube {
    fn new() -> Box<Actor> {
        Box::new(Cube {
            shadow_map: None,
            rotating: false,
            light: GameObject::empty(),
        })
    }

    fn start(&mut self, go: &mut GameObject, world: &mut World) {
        let db = &mut world.asset_system();

        let light_matrix = compute_light_matrix(&self.light);

        let material = Material::new(db.new_program("unrust/phong_shadow"));
        material.set("uMaterial.diffuse", db.new_texture("tex_a.png"));
        material.set("uMaterial.shininess", 32.0);
        material.set("uShadowMap", self.shadow_map.as_ref().unwrap().clone());
        material.set("uShadowMatrix", light_matrix);

        let mut mesh = Mesh::new();
        mesh.add_surface(db.new_mesh_buffer("cube"), material);
        go.add_component(mesh);
    }

    fn update(&mut self, go: &mut GameObject, _world: &mut World) {
        if self.rotating {
            let mut gtran = go.transform.global();
            gtran.append_rotation_wrt_center_mut(&UnitQuaternion::new(Vector3::new(
                0.01,
                0.02,
                0.005,
            )));
            go.transform.set_global(gtran);
        }
    }
}

pub struct Plane {
    shadow_map: Option<Rc<Texture>>,
    light: Handle<GameObject>,
}

impl Actor for Plane {
    fn new() -> Box<Actor> {
        Box::new(Plane {
            shadow_map: None,
            light: GameObject::empty(),
        })
    }

    fn start(&mut self, go: &mut GameObject, world: &mut World) {
        let db = &mut world.asset_system();
        let light_matrix = compute_light_matrix(&self.light);

        let material = Material::new(db.new_program("unrust/phong_shadow"));
        material.set("uMaterial.diffuse", db.new_texture("tex_a.png"));
        material.set("uMaterial.shininess", 32.0);
        material.set("uShadowMap", self.shadow_map.as_ref().unwrap().clone());
        material.set("uShadowMatrix", light_matrix);

        let mut mesh = Mesh::new();
        mesh.add_surface(db.new_mesh_buffer("plane"), material);
        go.add_component(mesh);
    }
}

pub fn main() {
    let mut world = WorldBuilder::new("Shadow demo")
        .with_size((800, 600))
        .with_stats(true)
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());
    drop(scene);

    world.event_loop();
}
