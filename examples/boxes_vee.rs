use na::{Translation3, Vector3};
use ncollide::shape::{Cuboid, Plane};
use nphysics3d::world::World;
use nphysics3d::object::{RigidBody, RigidBodyHandle};

pub struct Scene {
    pub world: World<f32>,
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
        let mut world = World::new();
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
