extern crate unrust;

use unrust::world::{Actor, Handle, World, WorldBuilder};
use unrust::engine::{AssetError, AssetSystem, Camera, Directional, GameObject, Light, Material,
                     Mesh, ObjMaterial, Prefab, RenderQueue, TextureWrap};
use unrust::world::events::*;
use unrust::actors::ShadowPass;
use unrust::math::*;

use std::rc::Rc;

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
            eye: Vector3::new(0.0, 0.36, -3.36),
            //eye: Vector3::new(12.0, 12.0, -12.0),
            last_event: None,
        })
    }
}

impl Actor for MainScene {
    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        // add main camera to scene
        {
            let go = world.new_game_object();
            let mut cam = Camera::default();
            cam.zfar = 300.0;
            go.borrow_mut().add_component(cam);
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

        // Added a simple plane
        {
            let plane = world.new_game_object();
            plane.borrow_mut().add_component(Plane::new_actor(Plane {}));
        }

        // Set the shadow map partitions
        {
            let shadow_pass = world.find_component::<ShadowPass>().unwrap();
            shadow_pass
                .borrow_mut()
                .set_partitions(&[5.0, 20.0, 50.0, 300.0]);
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
                            "KeyA" => self.eye = Quaternion::from_angle_y(Rad(0.2)) * self.eye,
                            "KeyD" => self.eye = Quaternion::from_angle_y(Rad(-0.2)) * self.eye,
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
        use imgui::TextAlign;

        imgui::pivot((1.0, 1.0));
        imgui::text_align(TextAlign::Right);
        imgui::label(
            Native(1.0, 1.0) - Pixel(8.0, 8.0 * 7.0),
            "[WASD] : control camera\n[Esc]  : reload all (include assets)",
        );

        imgui::pivot((1.0, 0.0));
        imgui::label(
            Native(1.0, 0.0) + Pixel(-8.0, 8.0),
            &format!("last event: {:?}", self.last_event),
        );

        imgui::pivot((1.0, 1.0));
        imgui::text_align(TextAlign::Right);
        imgui::label(
            Native(1.0, 1.0) - Pixel(8.0, 8.0),
            "Vending Machine by Don Carson\nhttps://poly.google.com/view/0CX6wj64Swu",
        );
    }
}

pub struct WaveObjActor {}

impl WaveObjActor {
    fn new() -> Box<Actor> {
        Box::new(WaveObjActor {})
    }
}

fn build_material(asys: &AssetSystem, obj_mat: ObjMaterial) -> Rc<Material> {
    let shader_program = match obj_mat.normal_map {
        Some(_) => asys.new_program("obj_nm"),
        None => asys.new_program("obj"),
    };

    let mut material = Material::new(shader_program);
    material.set(
        "uMaterial.ambient",
        obj_mat.ambient.unwrap_or(Vector3::new(0.2, 0.2, 0.2)),
    );

    let diffuse_tex = asys.new_texture(&obj_mat.diffuse_map.unwrap_or("default_white".to_owned()));
    diffuse_tex.wrap_u.set(TextureWrap::Repeat);
    diffuse_tex.wrap_v.set(TextureWrap::Repeat);
    material.set(
        "uMaterial.diffuse",
        obj_mat.diffuse.unwrap_or(Vector3::new(1.0, 1.0, 1.0)),
    );
    material.set("uMaterial.diffuse_tex", diffuse_tex);

    let specular_tex =
        asys.new_texture(&obj_mat.specular_map.unwrap_or("default_black".to_owned()));
    specular_tex.wrap_u.set(TextureWrap::Repeat);
    specular_tex.wrap_v.set(TextureWrap::Repeat);
    material.set(
        "uMaterial.specular",
        obj_mat.specular.unwrap_or(Vector3::new(1.0, 1.0, 1.0)),
    );
    material.set("uMaterial.specular_tex", specular_tex);

    material.set("uMaterial.shininess", obj_mat.shininess.unwrap_or(32.0));
    material.set("uMaterial.transparent", obj_mat.alpha.unwrap_or(1.0));

    obj_mat.normal_map.as_ref().map(|nm| {
        let n_tex = asys.new_texture(nm);
        n_tex.wrap_u.set(TextureWrap::Repeat);
        n_tex.wrap_v.set(TextureWrap::Repeat);

        material.set("uMaterial.normal_map", n_tex);
    });

    match obj_mat.alpha_mask {
        Some(ref f) => material.set("uMaterial.mask_tex", asys.new_texture(&f)),
        None => material.set("uMaterial.mask_tex", asys.new_texture("default_white")),
    }

    if obj_mat.alpha.unwrap_or(1.0) < 0.9999 || obj_mat.alpha_mask.is_some() {
        material.render_queue = RenderQueue::Transparent;
    }

    Rc::new(material)
}

impl Actor for WaveObjActor {
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

        db.new_prefab(
            "meshobj_test_model.obj",
            Box::new(build_material),
            Box::new(prefab_handler),
        );
    }
}

pub struct Plane;

impl Actor for Plane {
    fn start(&mut self, go: &mut GameObject, world: &mut World) {
        let db = &mut world.asset_system();

        let material = Material::new(db.new_program("unrust/phong_shadow"));
        material.set("uMaterial.diffuse", db.new_texture("default_white"));
        material.set("uMaterial.shininess", 32.0);

        let mut mesh = Mesh::new();
        mesh.add_surface(db.new_mesh_buffer("plane"), material);
        go.add_component(mesh);

        let mut ltran = go.transform.local();
        ltran.disp = Vector3::new(0.0, -1.45, 0.0);
        go.transform.set_local(ltran);
    }
}

pub fn main() {
    let mut world = WorldBuilder::new("obj prefab demo")
        .with_size((800, 600))
        .with_stats(true)
        .with_processor::<ShadowPass>()
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());
    drop(scene);

    world.event_loop();
}
