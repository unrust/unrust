#![feature(nll)]
#![feature(fnbox)]
#![recursion_limit = "512"]
#![feature(integer_atomics)]

/* common */
extern crate alga;
extern crate futures;
extern crate image;
extern crate obj;
extern crate uni_app;
extern crate uni_glsl;
extern crate webgl;

// reexport nalgebra
pub extern crate nalgebra as na;

pub mod engine;
pub mod world;

pub mod math {
    pub use na::{Point3, Rotation3, UnitQuaternion, Vector3};
}

pub use engine::imgui;
