#![feature(nll)]
#![recursion_limit = "512"]
#![feature(integer_atomics)]

/* common */
extern crate image;
extern crate nalgebra as na;
extern crate uni_app;
extern crate uni_glsl;
extern crate webgl;

#[macro_use]
extern crate lazy_static;

pub mod engine;
