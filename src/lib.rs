#![feature(nll)]
#![recursion_limit = "512"]
#![feature(integer_atomics)]

/* common */
extern crate futures;
extern crate image;
extern crate nalgebra as na;
extern crate tobj;
extern crate uni_glsl;
extern crate webgl;

pub mod engine;
