#![feature(nll)]
#![feature(fnbox)]
#![recursion_limit = "512"]
#![feature(integer_atomics)]
#![feature(conservative_impl_trait)]
#![cfg_attr(feature = "flame_it", feature(plugin, custom_attribute))]
#![cfg_attr(feature = "flame_it", plugin(flamer))]

/* common */
extern crate cgmath;
extern crate fnv;
extern crate futures;
extern crate image;
extern crate obj;
extern crate uni_app;
extern crate uni_glsl;
extern crate webgl;

#[macro_use]
extern crate bitflags;

#[cfg(feature = "flame_it")]
extern crate flame;

pub mod engine;
pub mod world;
pub mod actors;

pub mod math {
    pub use cgmath::prelude::*;
    pub use cgmath::*;
    pub use engine::Aabb;

    // pub fn transform_point(m: &Matrix4<f32>, p: &Point3<f32>) -> Point3<f32> {
    //     Point3::from_homogeneous(m * p.to_homogeneous())
    // }

    pub type Vector3f = Vector3<f32>;
    pub type Matrix4f = Matrix4<f32>;
    pub type Vector2f = Vector2<f32>;

    pub type Isometry3<T> = Decomposed<Vector3<T>, Quaternion<T>>;
}

pub use engine::imgui;
