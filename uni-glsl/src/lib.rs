#![feature(trace_macros)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate nom;

#[macro_use]
mod macros;

pub mod preprocessor;
mod token;

#[macro_use]
mod operator;
mod expression;
mod declaration;
mod statement;
mod parser;

pub use self::expression::{expression, Expression};
pub use self::declaration::{declaration, Declaration};
