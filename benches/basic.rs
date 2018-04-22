#![feature(test)]

extern crate test;
extern crate uni_pad;
extern crate unrust;

#[macro_use]
extern crate unrust_derive;

use test::Bencher;
use unrust::actors::FirstPersonCamera;
use unrust::engine::{Directional, GameObject, Light, Material, Mesh};
use unrust::math::*;
use unrust::world::{Actor, World, WorldBuilder};

// GUI
use unrust::imgui;

#[derive(Actor)]
pub struct MainScene {}

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
            go.borrow_mut().add_component(Cube{});
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

#[derive(Actor)]
pub struct Cube {}

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

#[bench]
fn bench_basic(b: &mut Bencher) {
    let mut world = WorldBuilder::new("Headless")
        .with_headless(true)
        .with_size((640, 480))
        .with_stats(true)
        .with_processor::<FirstPersonCamera>()  // Use first person camera
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene{});
    drop(scene);

    b.iter(move || {
        world.poll_events();
    });
}
