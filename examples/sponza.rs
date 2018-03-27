extern crate unrust;

use unrust::world::{Actor, Handle, Processor, World, WorldBuilder};
use unrust::engine::{AssetError, ComponentBased, Directional, GameObject, Light, Material, Point,
                     Prefab};
use unrust::world::events::*;
use unrust::math::*;
use unrust::actors::{FirstPersonCamera, ShadowPass, SkyBox};

// GUI
use unrust::imgui;

use std::rc::Rc;

pub struct MainScene {
    dir_light: Handle<GameObject>,
    point_light: Handle<GameObject>,

    animate_light: bool,
    last_event: Option<AppEvent>,
}

pub struct MaterialFilter {
    force_no_normal_map: bool,
}

impl ComponentBased for MaterialFilter {}
impl Actor for MaterialFilter {}

impl Processor for MaterialFilter {
    fn new() -> MaterialFilter {
        MaterialFilter {
            force_no_normal_map: false,
        }
    }

    fn apply_materials(&self, materials: &Vec<Rc<Material>>) {
        for m in materials.iter() {
            m.set("uNoNormalMap", self.force_no_normal_map);
        }
    }

    fn watch_material() -> bool
    where
        Self: Sized,
    {
        return true;
    }
}

// Actor is a trait object which would act like an component
// (Because Box<Actor> implemented ComponentBased)
impl MainScene {
    fn new() -> Box<Actor> {
        Box::new(MainScene {
            last_event: None,
            animate_light: true,
            dir_light: GameObject::empty(),
            point_light: GameObject::empty(),
        })
    }
}

impl Actor for MainScene {
    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        // add direction light to scene.
        {
            let go = world.new_game_object();
            let mut light = Light::new(Directional::default());
            light.directional_mut().unwrap().direction = Vector3f::new(-0.8, -1.0, 0.0).normalize();
            go.borrow_mut().add_component(light);

            self.dir_light = go;
        }

        {
            let go = world.new_game_object();
            go.borrow_mut().add_component(Light::new(Point::default()));
            self.point_light = go;
        }

        // Added the obj display
        {
            let go = world.new_game_object();
            go.borrow_mut().add_component(WaveObjActor::new());
        }

        let fpc_ref = world.find_component::<FirstPersonCamera>().unwrap();
        let mut fpc = fpc_ref.borrow_mut();

        fpc.camera().borrow_mut().znear = 1.0;
        fpc.camera().borrow_mut().zfar = 10000.0;

        fpc.eye = Vector3::new(0.0, 200.06, -3.36);
        fpc.eye_dir = Vector3::new(-3.0, 0.0, -1.0).normalize();
        fpc.speed = 8.0;

        {
            let shadow_pass = world.find_component::<ShadowPass>().unwrap();
            shadow_pass.borrow_mut().disable_cascaded();
        }
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        // Handle Events
        let normap_map_enabled;

        {
            let mut reset = false;
            let mut toggle_normal_map = false;

            for evt in world.events().iter() {
                self.last_event = Some(evt.clone());

                match evt {
                    &AppEvent::KeyUp(ref key) => match key.code.as_str() {
                        "KeyU" => toggle_normal_map = true,
                        "Space" => self.animate_light = !self.animate_light,
                        "Escape" => reset = true,

                        _ => (),
                    },

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

            if toggle_normal_map {
                let mf = world.find_component::<MaterialFilter>().unwrap();
                let b = mf.borrow().force_no_normal_map;
                mf.borrow_mut().force_no_normal_map = !b;
            }

            normap_map_enabled = {
                let mf = world.find_component::<MaterialFilter>().unwrap();
                let mf_borrow = mf.borrow();
                !mf_borrow.force_no_normal_map
            };
        }

        // Update Direction light
        if self.animate_light {
            let dir_light_bor = self.dir_light.borrow_mut();
            let (mut light, _) = dir_light_bor.find_component_mut::<Light>().unwrap();

            let dir = light.directional().unwrap().direction;
            let mut t = Isometry3::identity();
            t.append_rotation_mut(&UnitQuaternion::new(Vector3::new(0.0, 0.01, 0.0)));
            let transformed = t * dir;

            light.directional_mut().unwrap().direction = transformed;
        }

        // Update Point Light
        {
            let cam = world.current_camera().unwrap();

            let point_light_bor = self.point_light.borrow_mut();
            let (mut light, _) = point_light_bor.find_component_mut::<Light>().unwrap();
            light.point_mut().unwrap().linear = 0.0007;
            light.point_mut().unwrap().quadratic = 0.00002;
            light.point_mut().unwrap().position = cam.borrow().eye();
        }

        // GUI
        use imgui::Metric::*;
        use imgui::TextAlign::*;

        imgui::pivot((1.0, 1.0));
        imgui::text_align(Left);
        imgui::label(
            Native(1.0, 1.0) - Pixel(8.0, 8.0),
            "[WASD ZXEC] : control camera\n[Space] : Toggle light animation\n[U] : Toggle normal map\n[Esc] : reload all (include assets)",
        );

        imgui::pivot((1.0, 0.0));
        imgui::text_align(Right);
        imgui::label(
            Native(1.0, 0.0) + Pixel(-8.0, 8.0),
            &format!(
                "last event: {:?}\nnormal_map = {:?}\nlight animation={:?}",
                self.last_event, normap_map_enabled, self.animate_light
            ),
        );

        imgui::pivot((0.0, 1.0));
        imgui::text_align(Left);
        imgui::label(Native(0.0, 1.0) + Pixel(8.0, -8.0), "Sponza Demo");
    }
}

pub struct WaveObjActor {}

impl WaveObjActor {
    fn new() -> Box<Actor> {
        Box::new(WaveObjActor {})
    }
}

impl Actor for WaveObjActor {
    fn start_rc(&mut self, go: Handle<GameObject>, world: &mut World) {
        let db = &mut world.asset_system();

        let prefab_handler = {
            let go = go.clone();
            move |r: Result<Prefab, AssetError>| match r {
                Ok(prefab) => for c in prefab.components {
                    go.borrow_mut().add_component(c.clone());
                },
                Err(err) => {
                    panic!(format!("Cannot load prefab, reason:{:?}", err));
                }
            }
        };

        db.new_prefab("sponza/sponza.obj", Box::new(prefab_handler));
    }
}

pub fn main() {
    let mut world = WorldBuilder::new("sponza demo")
        .with_size((800, 600))
        .with_stats(true)
        .with_processor::<MaterialFilter>()
        .with_processor::<SkyBox>()
        .with_processor::<FirstPersonCamera>()
        .with_processor::<ShadowPass>()
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());
    drop(scene);

    world.event_loop();
}
