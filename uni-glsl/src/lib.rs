#![feature(trace_macros)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate nom;

#[macro_use]
mod macros;

#[macro_use]
mod operator;

mod token;
mod expression;
mod declaration;
mod statement;
mod defeval;

pub mod preprocessor;
pub mod parser;

pub use self::expression::{expression, Expression};
pub use self::declaration::{declaration, Declaration, FullyTypeSpecifier, FunctionPrototype,
                            ParamDeclaration, ParamQualifier, PrecisionQualifier, Struct,
                            StructMember, TypeQualifier, TypeSpecifier};
pub use self::statement::{statement, Statement};
