#![feature(nll)]
#![feature(fnbox)]
#![recursion_limit = "512"]
#![feature(integer_atomics)]
#![feature(conservative_impl_trait)]
#![cfg_attr(feature = "flame_it", feature(plugin, custom_attribute))]
#![cfg_attr(feature = "flame_it", plugin(flamer))]

/* common */
extern crate alga;
extern crate futures;
extern crate image;
extern crate obj;
extern crate uni_app;
extern crate uni_glsl;
extern crate webgl;

// reexport
pub extern crate nalgebra as na;
pub extern crate ncollide;
pub extern crate nphysics3d;

#[cfg(feature = "flame_it")]
extern crate flame;

pub mod engine;
pub mod world;
pub mod actors;

pub mod math {
    pub use na::{Isometry3, Matrix4, Point3, Rotation3, Translation3, UnitQuaternion, Vector2,
                 Vector3};
}

pub use engine::imgui;
