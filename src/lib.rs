#![feature(nll)]
#![feature(fnbox)]
#![recursion_limit = "512"]
#![feature(integer_atomics)]
#![cfg_attr(feature = "flame_it", feature(plugin, custom_attribute))]
#![cfg_attr(feature = "flame_it", plugin(flamer))]

/* common */
extern crate fnv;
extern crate futures;
extern crate hound;
extern crate image;
extern crate obj;
extern crate uni_app;
extern crate uni_glsl;
extern crate uni_pad;
extern crate uni_snd;
extern crate webgl;

#[macro_use]
extern crate unrust_derive;

#[macro_use]
extern crate bitflags;

#[cfg(feature = "flame_it")]
extern crate flame;

// This is here so that our procedural macros
// can work within the crate.
pub(crate) mod unrust {
    pub use super::*;
}


pub mod actors;
pub mod engine;
pub mod world;

pub mod math {
    pub extern crate cgmath;

    pub use self::cgmath::prelude::*;
    pub use self::cgmath::{ortho, vec3, Decomposed, Deg, Euler, Matrix3, Matrix4, PerspectiveFov,
                           Point3, Quaternion, Rad, Vector2, Vector3, Vector4};
    pub use engine::Aabb;

    pub type Vector3f = Vector3<f32>;
    pub type Matrix4f = Matrix4<f32>;
    pub type Vector2f = Vector2<f32>;

    pub type Isometry3<T> = Decomposed<Vector3<T>, Quaternion<T>>;
}

pub use engine::imgui;
