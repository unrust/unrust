extern crate nalgebra as na;
extern crate ncollide3d;
extern crate nphysics3d;
extern crate unrust;

#[macro_use]
extern crate unrust_derive;

use unrust::engine::{Camera, DirectionalLight, GameObject, Light, Material, Mesh, PointLight};
use unrust::math;
use unrust::math::Transform;
use unrust::world::events::*;
use unrust::world::{Actor, Handle, World, WorldBuilder};

use ncollide3d::shape::{Cuboid, Plane, ShapeHandle};
use nphysics3d::object;
use nphysics3d::object::{BodyHandle, ColliderHandle};
use nphysics3d::volumetric::Volumetric;
use nphysics3d::world::World as PhyWorld;

use std::cell::RefCell;
use std::rc::Rc;
use unrust::actors::{ShadowPass, SkyBox};

// GUI
use na::*;
use unrust::imgui;

pub struct Scene {
    pub world: Handle<PhyWorld<f32>>,
}

const COLLIDER_MARGIN: f32 = 0.01;

impl Scene {
    pub fn step(&mut self) {
        self.world.borrow_mut().step()
    }

    pub fn add_box(&mut self) -> ColliderHandle {
        let rad = 1.0;
        let geom = ShapeHandle::new(Cuboid::new(Vector3::new(
            rad - 0.04,
            rad - 0.04,
            rad - 0.04,
        )));
        let inertia = geom.inertia(1.0);
        let center_of_mass = geom.center_of_mass();

        let pos = Isometry3::new(Vector3::new(0.0, 30.0, 0.0), na::zero());
        let handle = self.world
            .borrow_mut()
            .add_rigid_body(pos, inertia, center_of_mass);

        self.world.borrow_mut().add_collider(
            COLLIDER_MARGIN,
            geom.clone(),
            handle,
            Isometry3::identity(),
            object::Material::default(),
        )
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
        let ground_shape =
            ShapeHandle::new(Plane::new(Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0))));

        world.add_collider(
            COLLIDER_MARGIN,
            ground_shape,
            BodyHandle::ground(),
            Isometry3::identity(),
            object::Material::default(),
        );
        /*
         * Create the boxes
         */
        let num = 4;
        let rad = 1.0;
        let shift = rad * 2.0;
        let centerx = shift * (num as f32) / 2.0;
        let centery = shift / 2.0 + 0.04;
        let centerz = shift * (num as f32) / 2.0;

        let geom = ShapeHandle::new(Cuboid::new(Vector3::repeat(rad - COLLIDER_MARGIN)));
        let inertia = geom.inertia(1.0);
        let center_of_mass = geom.center_of_mass();

        for i in 0usize..num {
            for j in 0usize..num {
                for k in 0usize..num {
                    let x = i as f32 * shift - centerx;
                    let y = j as f32 * shift + centery;
                    let z = k as f32 * shift - centerz;

                    let pos = Isometry3::new(Vector3::new(x, y, z), na::zero());
                    let handle = world.add_rigid_body(pos, inertia, center_of_mass);

                    /*
                     * Create the collider.
                     */
                    world.add_collider(
                        COLLIDER_MARGIN,
                        geom.clone(),
                        handle,
                        Isometry3::identity(),
                        object::Material::default(),
                    );
                }
            }
        }

        Scene {
            world: Rc::new(RefCell::new(world)),
        }
    }
}

// Physic Object Component
#[derive(Component)]
struct PhysicObject(ColliderHandle, Handle<PhyWorld<f32>>);

impl PhysicObject {
    fn phy_transform(&self) -> math::Isometry3<f32> {
        let world = self.1.borrow();
        let collider = world.collider(self.0).unwrap();
        let body = match world.body(collider.data().body()) {
            object::Body::RigidBody(rb) => rb,
            _ => return math::Isometry3::one(),
        };

        let na_pos = body.position();

        use unrust::math::InnerSpace;

        math::Isometry3 {
            scale: 1.0,
            rot: math::Quaternion::new(
                na_pos.rotation.coords.w,
                na_pos.rotation.coords.x,
                na_pos.rotation.coords.y,
                na_pos.rotation.coords.z,
            ).normalize(),
            disp: math::Vector3::new(
                na_pos.translation.vector.x,
                na_pos.translation.vector.y,
                na_pos.translation.vector.z,
            ),
        }
    }
}

#[derive(Actor)]
pub struct MainScene {
    eye: math::Vector3<f32>,
    last_event: Option<AppEvent>,
    phy_scene: Scene,
    counter: u32,
    point_lights: Vec<Handle<GameObject>>,
}

impl MainScene {
    fn colliders(&mut self) -> Vec<ColliderHandle> {
        self.phy_scene
            .world
            .borrow()
            .colliders()
            .map(|cb| cb.handle())
            .collect()
    }

    fn add_box(&mut self, world: &mut World) {
        let bx = self.phy_scene.add_box();
        self.add_object(bx, world);
    }

    fn add_object(&mut self, id: ColliderHandle, world: &mut World) {
        let phy_world = self.phy_scene.world.borrow();
        let collider = phy_world.collider(id).unwrap();
        let shape = collider.shape();

        let go = world.new_game_object();
        go.borrow_mut()
            .add_component(PhysicObject(id, self.phy_scene.world.clone()));

        if let Some(_) = shape.as_shape::<Cuboid<f32>>() {
            let actor = CubeActor { id: self.counter };
            self.counter += 1;
            go.borrow_mut().add_component(actor);
        } else if let Some(_) = shape.as_shape::<Plane<f32>>() {
            go.borrow_mut().add_component(PlaneActor {});
        } else {
            unimplemented!();
        }
    }

    fn new() -> Self {
        MainScene {
            eye: math::Vector3::new(26.0, 38.0, -43.0),
            last_event: None,
            phy_scene: Scene::new(),
            point_lights: Vec::new(),
            counter: 0,
        }
    }
}

// Actor is a trait object which would act like an component
// (Because Box<Actor> is Component)
impl Actor for MainScene {
    fn start(&mut self, _: &mut GameObject, world: &mut World) {
        // add main camera to scene
        {
            let go = world.new_game_object();
            go.borrow_mut().add_component(Camera::default());
        }

        // add direction light to scene.
        {
            let go = world.new_game_object();
            go.borrow_mut().add_component(DirectionalLight::default());
        }

        // add points light

        // Add 4 points light to scene
        let point_light_positions = vec![
            math::Vector3::new(-30.0, 30.0, -30.0),
            math::Vector3::new(-15.0, 300.0, -10.0),
            math::Vector3::new(30.0, 50.0, 30.0),
            math::Vector3::new(30.0, 100.0, -20.0),
        ];

        for p in point_light_positions.into_iter() {
            let go = world.new_game_object();
            let mut point_light = PointLight::default();
            point_light.position = p;
            go.borrow_mut().add_component(point_light);

            self.point_lights.push(go.clone());
        }

        // Add the physics object
        {
            let colliders = self.colliders();

            for cb in colliders.into_iter() {
                self.add_object(cb, world);
            }
        }
    }

    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        use unrust::math::{EuclideanSpace, InnerSpace, Rotation3};

        self.phy_scene.step();

        // Update point lights
        for lgo in self.point_lights.iter() {
            lgo.try_borrow().ok().map(|light_go| {
                if let Some((ref mut light, _)) = light_go.find_component_mut::<Light>() {
                    let mut pos = light.point().unwrap().position;

                    light.point_mut().unwrap().position =
                        math::Quaternion::from_angle_y(math::Rad(0.02)) * pos;
                }
            });
        }

        // Handle Events
        {
            let target = math::Vector3::new(0.0, 0.0, 0.0);
            let front = (self.eye - target).normalize();

            let mut reset = false;
            let mut addbox = false;

            for evt in world.events().iter() {
                self.last_event = Some(evt.clone());
                match evt {
                    &AppEvent::MouseUp(_) => addbox = true,
                    &AppEvent::KeyDown(ref key) => {
                        match key.code.as_str() {
                            "KeyA" => {
                                self.eye =
                                    math::Quaternion::from_angle_y(math::Rad(-0.02)) * self.eye
                            }
                            "KeyD" => {
                                self.eye =
                                    math::Quaternion::from_angle_y(math::Rad(0.02)) * self.eye
                            }
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
                &math::Point3::from_vec(self.eye),
                &math::Point3::new(0.0, 0.0, 0.0),
                &math::Vector3::new(0.0, 1.0, 0.0),
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

#[derive(Actor)]
pub struct CubeActor {
    id: u32,
}

impl Actor for CubeActor {
    fn start(&mut self, go: &mut GameObject, world: &mut World) {
        let db = &mut world.asset_system();

        let texture = match self.id % 5 {
            0 => db.new_texture("tex_a.png"),
            1 => db.new_texture("tex_r.png"),
            _ => db.new_texture("tex_b.png"),
        };

        let material = Material::new(db.new_program("unrust/phong_shadow"));
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

#[derive(Actor)]
pub struct PlaneActor {}

impl Actor for PlaneActor {
    fn start(&mut self, go: &mut GameObject, world: &mut World) {
        let db = &mut world.asset_system();

        let material = Material::new(db.new_program("unrust/phong_shadow"));
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
        go.transform
            .set_local_scale(math::Vector3::new(3.0, 1.0, 3.0));
    }
}

pub fn main() {
    let mut world = WorldBuilder::new("Boxes with physics demo")
        .with_size((800, 600))
        .with_stats(true)
        .with_processor::<ShadowPass>()
        .with_processor::<SkyBox>()
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(MainScene::new());

    drop(scene);

    world.event_loop();
}
