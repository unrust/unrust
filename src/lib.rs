#![feature(nll)]
#![feature(fnbox)]
#![recursion_limit = "512"]
#![feature(integer_atomics)]

/* common */
extern crate alga;
extern crate futures;
extern crate image;
extern crate nalgebra as na;
extern crate obj;
extern crate uni_glsl;
extern crate webgl;

pub mod engine;
