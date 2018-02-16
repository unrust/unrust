#![allow(unused_imports)]

use nom::types::CompleteStr;
use nom::{digit, hex_digit, oct_digit, recognize_float, sp, space, Err, IResult};
use nom::line_ending;
use std::fmt::Debug;
use std::fmt;
use std::convert::From;
use std::str::FromStr;
use std::collections::HashMap;
use std::error;

use super::operators::{operator, Operator};

type CS<'a> = CompleteStr<'a>;

#[derive(Clone, PartialEq)]
pub enum Constant {
    Bool(bool),
    Integer(i64),
    Float(f32),
}

macro_rules! impl_constant_from {
    ($t:ty, $i:ident) => {
        impl From<$t> for Constant {
            fn from(i: $t) -> Self {
                Constant::$i(i)
            }
        }
    }
}

impl_constant_from!(i64, Integer);
impl_constant_from!(bool, Bool);
impl_constant_from!(f32, Float);

impl Constant {
    fn from<T>(v: T) -> Self
    where
        T: Into<Self>,
    {
        v.into()
    }
}

impl Debug for Constant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Constant::Integer(ref s) => write!(f, "Constant::Integer {{ {:?} }}", s),
            &Constant::Float(ref s) => write!(f, "Constant::Float {{ {:?} }}", s),
            &Constant::Bool(ref b) => write!(f, "Constant::Bool {{ {:?} }}", b),
        }
    }
}

macro_rules! spe {
  ($i:expr, $($args:tt)*) => {{
    delimited!($i, opt!(space), $($args)*, opt!(space))
  }}
}

pub type Identifier = String;

/* Parser */
#[derive(Clone, PartialEq)]
pub enum Token {
    Operator(Operator, String),
    Constant(Constant, String),
    BasicType(BasicType, String),
    Identifier(Identifier, String),
}

impl Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Token::Operator(ref s, _) => write!(f, "Token::Operator {{ {:?} }}", s),
            &Token::Identifier(ref s, _) => write!(f, "Token::Identifier {{ {:?} }}", s),
            &Token::Constant(ref n, _) => write!(f, "Token::Constant {{ {:?} }}", n),
            &Token::BasicType(_, ref s) => write!(f, "Token::BasicType {{ {:?} }}", s),
        }
    }
}

impl Token {
    fn from_identifier((s, cs): (Identifier, CompleteStr)) -> Token {
        Token::Identifier(s, cs.0.into())
    }

    fn from_operator((s, cs): (Operator, CompleteStr)) -> Token {
        Token::Operator(s, cs.0.into())
    }

    fn from_constant((n, cs): (Constant, CompleteStr)) -> Token {
        Token::Constant(n, cs.0.into())
    }

    fn from_basic_type((n, cs): (BasicType, CompleteStr)) -> Token {
        Token::BasicType(n, cs.0.into())
    }
}

/// identifier macro
named!(
    pub identifier<CS, Identifier>,
    map!(do_parse!(
        name: verify!(take_while1!(|ch:char|ch.is_alphanumeric() || ch == '_'), verify_identifier) >> (name)
    ), |cs| cs.0.into() )
);

#[inline]
fn verify_identifier(s: CompleteStr) -> bool {
    match s.0.chars().next() {
        Some(ref c) => !c.is_digit(10),
        None => false,
    }
}

named!(non_zero_digit<CS,char>, one_of!("123456789"));

/// integer parsers
named!(
    decimal_constant<CS, i64>,
    map_res!(recognize!( do_parse!(
        non_zero_digit >>
        opt!(digit) >>
        ()
    )), |cs:CS| FromStr::from_str(cs.0) )    
);

named!(
    octal_constant<CS, i64>,
    map_res!(recognize!( do_parse!(
        tag!("0") >>
        opt!(oct_digit) >>
        ()
    )), |cs:CS| FromStr::from_str(cs.0) )
);

named!(
    hexadecimal_constant<CS, i64>,
    map_res!(recognize!( do_parse!(
        tag_no_case!("0x") >>
        hex_digit >>
        ()
    )), |cs:CS| i64::from_str_radix(&cs.0[2..], 16))
);

named!(
    pub integer_constant<CS, i64>,
    alt_complete!(hexadecimal_constant|decimal_constant|octal_constant)
);

/// float constant parser
named!(
    pub float_constant<CS, f32>,
    map_res!(recognize_float, |cs:CS| FromStr::from_str(cs.0) )
);

/// bool constant parser
named!(
    pub bool_constant<CS, bool>,
    alt!(value!(true, tag!("true")) | value!(false, tag!("false")))
);

/// Constant macro
named!(
    pub constant<CS, Constant>,
    alt_complete!(
        map!(integer_constant, Constant::from) |
        map!(float_constant, Constant::from) |        
        map!(bool_constant, Constant::from)
    )
);

#[derive(Clone, Debug, PartialEq)]
pub enum BasicType {
    Void,
    Bool,
    Int,
    Float,
    Vec2,
    Vec3,
    Vec4,
    Bvec2,
    Bvec3,
    Bvec4,
    Ivec2,
    Ivec3,
    Ivec4,
    Mat2,
    Mat3,
    Mat4,
    Sampler2D,
    SamplerCube,
    TypeName(Identifier),
}

named!(pub basic_type<CS,BasicType>,
    alt!(
        value!(BasicType::Void, tag!("void")) |
        value!(BasicType::Bool, tag!("boid")) |
        value!(BasicType::Int, tag!("int")) |
        value!(BasicType::Float, tag!("float")) |
        value!(BasicType::Vec2, tag!("Vec2")) |
        value!(BasicType::Vec3, tag!("Vec3")) |
        value!(BasicType::Vec4, tag!("Vec4")) |
        value!(BasicType::Bvec2, tag!("Bvec2")) |
        value!(BasicType::Bvec3, tag!("Bvec3")) |
        value!(BasicType::Bvec4, tag!("Bvec4")) |
        value!(BasicType::Ivec2, tag!("Ivec2")) |
        value!(BasicType::Ivec3, tag!("Ivec3")) |
        value!(BasicType::Ivec4, tag!("Ivec4")) |
        value!(BasicType::Mat2, tag!("Mat3")) |
        value!(BasicType::Mat3, tag!("Mat3")) |
        value!(BasicType::Mat4, tag!("Mat4")) |
        value!(BasicType::Sampler2D, tag!("sampler2D")) |
        value!(BasicType::SamplerCube, tag!("sampler3D"))
    )
);

#[macro_export]
macro_rules! value_text {
  ($i:expr, $($args:tt)*) => {{
    do_parse!($i,
        s : peek!(recognize!($($args)*)) >>
        v : $($args)* >>
        (v,s)
    )
  }}
}

/// token macro
named!(pub token<CS, Token>, do_parse!(
    tt: spe!(alt!(
        map!( value_text!(operator), Token::from_operator) |   
        map!( value_text!(constant),Token::from_constant) |
        map!( value_text!(basic_type), Token::from_basic_type) |        
        map!( value_text!(identifier), Token::from_identifier)         
    )) >> 
    (tt)
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_integer() {
        // dec integer
        let i = integer_constant(CompleteStr("123456"));
        assert_eq!(i, Ok((CompleteStr(""), 123456i64)));

        // // oct integer
        let i = integer_constant(CompleteStr("0123456"));
        assert_eq!(i, Ok((CompleteStr(""), 0123456)));

        // hex integer
        let i = integer_constant(CompleteStr("0x1f4Fa"));
        assert_eq!(i, Ok((CompleteStr(""), 0x1f4Fa)));
    }
}
