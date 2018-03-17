#![feature(nll)]
#![feature(fnbox)]
#![recursion_limit = "512"]
#![feature(integer_atomics)]
#![feature(conservative_impl_trait)]
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
extern crate bitflags;

#[cfg(feature = "flame_it")]
extern crate flame;

pub mod engine;
pub mod world;
pub mod actors;

pub mod math {
    pub extern crate cgmath;

    pub use self::cgmath::prelude::*;
    pub use engine::Aabb;
    pub use self::cgmath::{ortho, Decomposed, Deg, Euler, Matrix3, Matrix4, PerspectiveFov,
                           Point3, Quaternion, Rad, Vector2, Vector3, Vector4, vec3};

    pub type Vector3f = Vector3<f32>;
    pub type Matrix4f = Matrix4<f32>;
    pub type Vector2f = Vector2<f32>;

    pub type Isometry3<T> = Decomposed<Vector3<T>, Quaternion<T>>;
}

pub use engine::imgui;
