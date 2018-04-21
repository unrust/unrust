extern crate image;
extern crate uni_pad;
extern crate unrust;

use std::env;
use std::fmt;
use std::path::PathBuf;
use unrust::actors::FirstPersonCamera;
use unrust::engine::{Directional, GameObject, Light, Material, Mesh};
use unrust::math::*;
use unrust::world::{Actor, World, WorldBuilder};

// GUI
use unrust::imgui;

pub struct MainScene {}

// Actor is a trait object which would act like an component
// (Because Box<Actor> implemented ComponentBased)
impl MainScene {
    fn new() -> Box<Actor> {
        Box::new(MainScene {})
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

    fn update(&mut self, _go: &mut GameObject, _: &mut World) {
        // GUI
        use imgui::Metric::*;

        imgui::pivot((1.0, 0.0));
        imgui::label(Native(1.0, 0.0) + Pixel(-8.0, 8.0), "Testing");
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

fn is_golden() -> bool {
    match env::var("UNRUST_TEST_GOLDEN") {
        Ok(golden) => golden == "1",
        _ => false,
    }
}

struct Image<'a>(&'a image::RgbaImage);

impl<'a> fmt::Debug for Image<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Image {{ width: {}, height: {} }}",
            self.0.width(),
            self.0.height()
        )
    }
}

impl<'a> PartialEq for Image<'a> {
    fn eq(&self, b: &Self) -> bool {
        if self.0.dimensions() != b.0.dimensions() {
            return false;
        }

        let (w, h) = self.0.dimensions();

        for x in 0..w {
            for y in 0..h {
                if self.0.get_pixel(x, y) != b.0.get_pixel(x, y) {
                    return false;
                }
            }
        }

        true
    }
}

#[test]
fn test_basic() {
    let mut world = WorldBuilder::new("Headless")
        .with_headless(true)
        .with_size((640, 480))
        .with_processor::<FirstPersonCamera>()  // Use first person camera
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());
    drop(scene);

    // We try to render 100 frames
    for _ in 0..100 {
        if !world.poll_events() {
            break;
        }
    }

    let img = world.engine().capture_frame_buffer();

    let mut golden_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    golden_dir.push("tests");
    golden_dir.push("resources");
    let golden_path = golden_dir.join("basic_golden.png");
    let img = img.expect("Cannot capture frame buffer");

    if is_golden() {
        img.save(golden_path).expect("Cannot save to file");
    } else {
        let golden = image::open(golden_path).unwrap();
        // For test fail
        // if Image(&golden.to_rgba()) != Image(&img) {
        //     img.save(golden_dir.join("basic_fail.png"))
        //         .expect("Cannot save to file");
        // }

        assert_eq!(
            Image(&golden.to_rgba()),
            Image(&img),
            "Image is not the same."
        );
    }
}
