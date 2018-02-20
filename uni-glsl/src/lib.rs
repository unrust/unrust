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
mod statement;
mod defeval;
mod declaration;

pub mod preprocessor;
pub mod parser;
pub mod query;

pub use self::expression::{expression, Expression};
pub use self::declaration::{declaration, Declaration, FullyTypeSpecifier, FunctionPrototype,
                            ParamDeclaration, ParamQualifier, PrecisionQualifier,
                            SingleDeclaration, Struct, StructMember, TypeQualifier, TypeSpecifier,
                            VariantTypeSpecifier};
pub use self::statement::{statement, Statement};
pub use self::parser::TranslationUnit;
pub use self::token::BasicType;
