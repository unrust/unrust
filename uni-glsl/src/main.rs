#![feature(trace_macros)]

#[macro_use]
extern crate nom;

mod preprocessor;
mod tokens;
mod operators;
mod parser;

fn main() {
    use std::fs::File;
    use std::io::prelude::*;

    let mut file = File::open("data/test/phong_fs.glsl").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    println!("{}", preprocessor::preprocess(&contents).unwrap());
}
