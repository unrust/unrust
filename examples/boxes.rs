extern crate unrust;

use unrust::world::{Actor, Handle, World, WorldBuilder};
use unrust::engine::{Camera, Directional, GameObject, Light, Material, Mesh, Point};
use unrust::world::events::*;
use unrust::math::*;

use unrust::ncollide::shape::{Cuboid, Cuboid3, Plane, Plane3};
use unrust::nphysics3d::world::World as PhyWorld;
use unrust::nphysics3d::object::{RigidBody, RigidBodyHandle};

use unrust::engine::ComponentBased;

// GUI
use unrust::imgui;

pub struct Scene {
    pub world: PhyWorld<f32>,
}

impl Scene {
    pub fn step(&mut self) {
        self.world.step(0.016)
    }

    pub fn add_box(&mut self) -> RigidBodyHandle<f32> {
        let rad = 1.0;
        let geom = Cuboid::new(Vector3::new(rad - 0.04, rad - 0.04, rad - 0.04));
        let mut rb = RigidBody::new_dynamic(geom, 1.0, 0.3, 0.5);
        rb.append_translation(&Translation3::new(0.0, 30.0, 0.0));

        self.world.add_rigid_body(rb)
    }

    pub fn new() -> Scene {
        /*
         * World
         */
        let mut world = PhyWorld::new();
        world.set_gravity(Vector3::new(0.0, -9.81, 0.0));

        /*
         * Plane
         */
        let geom = Plane::new(Vector3::new(0.0, 1.0, 0.0));

        world.add_rigid_body(RigidBody::new_static(geom, 0.3, 0.6));

        /*
         * Create the boxes
         */
        let num = 4;
        let rad = 1.0;
        let shift = rad * 2.0;
        let centerx = shift * (num as f32) / 2.0;
        let centery = shift / 2.0 + 0.04;
        let centerz = shift * (num as f32) / 2.0;

        for i in 0usize..num {
            for j in 0usize..num {
                for k in 0usize..num {
                    let x = i as f32 * shift - centerx;
                    let y = j as f32 * shift + centery;
                    let z = k as f32 * shift - centerz;

                    let geom = Cuboid::new(Vector3::new(rad - 0.04, rad - 0.04, rad - 0.04));
                    let mut rb = RigidBody::new_dynamic(geom, 1.0, 0.3, 0.5);

                    rb.append_translation(&Translation3::new(x, y, z));

                    world.add_rigid_body(rb);
                }
            }
        }

        Scene { world: world }
    }
}

// Physic Object Component
struct PhysicObject(Handle<RigidBody<f32>>);
impl PhysicObject {
    fn phy_transform(&self) -> Isometry3<f32> {
        *self.0.borrow().position()
    }
}
impl ComponentBased for PhysicObject {}

pub struct MainScene {
    eye: Vector3<f32>,
    last_event: Option<AppEvent>,
    phy_scene: Scene,
    counter: u32,
    point_lights: Vec<Handle<GameObject>>,
}

impl MainScene {
    fn rigid_bodies(&mut self) -> Vec<Handle<RigidBody<f32>>> {
        self.phy_scene
            .world
            .rigid_bodies()
            .map(|rb| rb.clone())
            .collect()
    }

    fn add_box(&mut self, world: &mut World) {
        let bx = self.phy_scene.add_box();
        self.add_object(bx, world);
    }

    fn add_object(&mut self, rb: Handle<RigidBody<f32>>, world: &mut World) {
        let rbody = rb.borrow();
        let shape = rbody.shape();

        let go = world.new_game_object();
        go.borrow_mut().add_component(PhysicObject(rb.clone()));

        if let Some(_) = shape.as_shape::<Cuboid3<f32>>() {
            let actor = CubeActor::new_actor(CubeActor { id: self.counter });
            self.counter += 1;
            go.borrow_mut().add_component(actor);
        } else if let Some(_) = shape.as_shape::<Plane3<f32>>() {
            go.borrow_mut().add_component(PlaneActor::new());
        } else {
            unimplemented!();
        }
    }
}

// Actor is a trait object which would act like an component
// (Because Box<Actor> implemented ComponentBased)
impl Actor for MainScene {
    fn new() -> Box<Actor> {
        Box::new(MainScene {
            eye: Vector3::new(-30.0, 30.0, -30.0),
            last_event: None,
            phy_scene: Scene::new(),
            point_lights: Vec::new(),
            counter: 0,
        })
    }

    fn start(&mut self, _: &mut GameObject, world: &mut World) {
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

        // add points light

        // Add 4 points light to scene
        let point_light_positions = vec![
            Vector3::new(-30.0, 30.0, -30.0),
            Vector3::new(-15.0, 300.0, -10.0),
            Vector3::new(30.0, 50.0, 30.0),
            Vector3::new(30.0, 100.0, -20.0),
        ];

        for p in point_light_positions.into_iter() {
            let go = world.new_game_object();
            let mut point_light = Point::default();
            point_light.position = p;
            go.borrow_mut().add_component(Light::new(point_light));

            self.point_lights.push(go.clone());
        }

        // Add the physics object
        {
            let rigid_bodies = self.rigid_bodies();

            for rb in rigid_bodies.into_iter() {
                self.add_object(rb, world);
            }
        }
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        self.phy_scene.step();

        // Update point lights
        for lgo in self.point_lights.iter() {
            lgo.try_borrow().ok().map(|light_go| {
                if let Some((ref mut light, _)) = light_go.find_component_mut::<Light>() {
                    let mut pos = light.point().unwrap().position;
                    light.point_mut().unwrap().position = Rotation3::new(Vector3::y() * 0.02) * pos;
                }
            });
        }

        // Handle Events
        {
            let target = Vector3::new(0.0, 0.0, 0.0);
            let front = (self.eye - target).normalize();
            let up = Vector3::y();

            let mut reset = false;
            let mut addbox = false;

            for evt in world.events().iter() {
                self.last_event = Some(evt.clone());
                match evt {
                    &AppEvent::Click(_) => addbox = true,
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

            if addbox {
                self.add_box(world);
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
            "Click on canvas to drop new box.\n[WASD] : control camera\n[Esc]  : reload all (include assets)",
        );

        imgui::pivot((1.0, 0.0));
        imgui::label(
            Native(1.0, 0.0) + Pixel(-8.0, 8.0),
            &format!("last event: {:?}", self.last_event),
        );
    }
}

pub struct CubeActor {
    id: u32,
}

impl Actor for CubeActor {
    fn new() -> Box<Actor> {
        Box::new(CubeActor { id: 0 })
    }

    fn start(&mut self, go: &mut GameObject, world: &mut World) {
        let db = &mut world.asset_system();

        let texture = match self.id % 5 {
            0 => db.new_texture("tex_a.png"),
            1 => db.new_texture("tex_r.png"),
            _ => db.new_texture("tex_b.png"),
        };

        let material = Material::new(db.new_program("phong"));
        material.set("uMaterial.diffuse", texture);
        material.set("uMaterial.shininess", 32.0);

        let mut mesh = Mesh::new();
        mesh.add_surface(db.new_mesh_buffer("cube"), material);
        go.add_component(mesh);
    }

    fn update(&mut self, go: &mut GameObject, _world: &mut World) {
        let new_trans = {
            let (phy, _) = go.find_component::<PhysicObject>().unwrap();
            phy.phy_transform()
        };

        go.transform.set_global(new_trans);
    }
}

pub struct PlaneActor {}

impl Actor for PlaneActor {
    fn new() -> Box<Actor> {
        Box::new(PlaneActor {})
    }

    fn start(&mut self, go: &mut GameObject, world: &mut World) {
        let db = &mut world.asset_system();

        let material = Material::new(db.new_program("phong"));
        material.set("uMaterial.diffuse", db.new_texture("tex_a.png"));
        material.set("uMaterial.shininess", 32.0);

        let mut mesh = Mesh::new();
        mesh.add_surface(db.new_mesh_buffer("plane"), material);
        go.add_component(mesh);
    }

    fn update(&mut self, go: &mut GameObject, _world: &mut World) {
        let new_trans = {
            let (phy, _) = go.find_component::<PhysicObject>().unwrap();
            phy.phy_transform()
        };

        go.transform.set_global(new_trans);
    }
}

pub fn main() {
    let mut world = WorldBuilder::new("Boxes with physics demo")
        .with_size((800, 600))
        .with_stats(true)
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());
    drop(scene);

    world.event_loop();
}
