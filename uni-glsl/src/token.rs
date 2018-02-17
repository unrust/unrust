use nom::types::CompleteStr;
use nom::{digit, hex_digit, oct_digit, recognize_float, space};
use std::str;
use super::operator::{operator, Operator};
use declaration::Struct;
use std::collections::HashSet;

type CS<'a> = CompleteStr<'a>;

#[derive(Clone, PartialEq, Debug)]
pub enum Constant {
    Bool(bool),
    Integer(i64),
    Float(f32),
}

macro_rules! spe {
  ($i:expr, $($args:tt)*) => {{
    delimited!($i, opt!(space), $($args)*, opt!(space))
  }}
}

pub type Identifier = String;

/* Parser */
#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    Operator(Operator, String),
    Constant(Constant, String),
    BasicType(BasicType, String),
    Identifier(Identifier, String),
}

/// identifier macro
named!(
    pub identifier<CS, Identifier>,
    map!(do_parse!(
        name: verify!(take_while1!(|ch:char|ch.is_alphanumeric() || ch == '_'), verify_identifier) >> (name)
    ), |cs| cs.0.into() )
);

/// non keyword identifier macro
named!(
    pub valid_name<CS, Identifier>,
    map!(do_parse!(
        name: verify!(take_while1!(|ch:char|ch.is_alphanumeric() || ch == '_'), verify_valid_name) >> (name)
    ), |cs| cs.0.into() )
);

lazy_static! {
    static ref KEYWORDS: HashSet<&'static str> = {
        let mut m = HashSet::new();
        m.insert("attribute");
        m.insert("const");
        m.insert("uniform");
        m.insert("varying");
        m.insert("break");
        m.insert("continue");
        m.insert("do");
        m.insert("for");
        m.insert("while");
        m.insert("if");
        m.insert("else");
        m.insert("in");
        m.insert("out");
        m.insert("inout");
        m.insert("float");
        m.insert("int");
        m.insert("void");
        m.insert("bool");
        m.insert("true");
        m.insert("false");
        m.insert("lowp");
        m.insert("mediump");
        m.insert("highp");
        m.insert("precision");
        m.insert("invariant");
        m.insert("discard");
        m.insert("return");
        m.insert("mat2");
        m.insert("mat3");
        m.insert("mat4");
        m.insert("vec2");
        m.insert("vec3");
        m.insert("vec4");
        m.insert("ivec2");
        m.insert("ivec3");
        m.insert("ivec4");
        m.insert("bvec2");
        m.insert("bvec3");
        m.insert("bvec4");
        m.insert("sampler2D");
        m.insert("samplerCube");
        m.insert("struct");
        m.insert("asm");
        m.insert("class");
        m.insert("union");
        m.insert("enum");
        m.insert("typedef");
        m.insert("template");
        m.insert("this");
        m.insert("packed");

        m.insert("goto");
        m.insert("switch");
        m.insert("default");
        m.insert("inline");
        m.insert("noinline");
        m.insert("volatile"   );
        m.insert("public");
        m.insert("static");
        m.insert("extern");
        m.insert("external");
        m.insert("interface");
        m.insert("flat");
        m.insert("long");
        m.insert("short");
        m.insert("double");
        m.insert("half");
        m.insert("fixed");
        m.insert("unsigned");
        m.insert("superp");
        m.insert("input");
        m.insert("output");
        m.insert("hvec2");
        m.insert("hvec3");
        m.insert("hvec4");
        m.insert("dvec2");
        m.insert("dvec3");
        m.insert("dvec4");
        m.insert("fvec2");
        m.insert("fvec3");
        m.insert("fvec4");
        m.insert("sampler1D");
        m.insert("sampler3D");
        m.insert("sampler1DShadow");
        m.insert("sampler2DShadow");
        m.insert("sampler2DRect");
        m.insert("sampler3DRect");
        m.insert("sampler2DRectShadow");
        m.insert("sizeof");
        m.insert("cast");
        m.insert("namespace");
        m.insert("using");
        m
    };
}

fn verify_valid_name(s: CompleteStr) -> bool {
    if !verify_identifier(s) {
        return false;
    }
    !KEYWORDS.contains(s.0)
}

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
    )), |cs:CS| str::FromStr::from_str(cs.0) )    
);

named!(
    octal_constant<CS, i64>,
    map_res!(recognize!( do_parse!(
        tag!("0") >>
        opt!(oct_digit) >>
        ()
    )), |cs:CS| str::FromStr::from_str(cs.0) )
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
    map_res!(recognize_float, |cs:CS| str::FromStr::from_str(cs.0) )
);

/// bool constant parser
named!(
    #[allow(unused_imports)], // fix value! warning
    pub bool_constant<CS, bool>,
    alt!(value!(true, tag!("true")) | value!(false, tag!("false")))
);

/// Constant macro
named!(
    pub constant<CS, Constant>,
    alt_complete!(
        map!(integer_constant, Constant::Integer) |
        map!(float_constant, Constant::Float) |        
        map!(bool_constant, Constant::Bool)
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
    Struct(Struct),
    TypeName(Identifier),
}

named!(
    #[allow(unused_imports)], // fix value! warning
    pub basic_type<CS,BasicType>,
    alt!(
        value!(BasicType::Void, tag!("void")) |
        value!(BasicType::Bool, tag!("boid")) |
        value!(BasicType::Int, tag!("int")) |
        value!(BasicType::Float, tag!("float")) |
        value!(BasicType::Vec2, tag!("vec2")) |
        value!(BasicType::Vec3, tag!("vec3")) |
        value!(BasicType::Vec4, tag!("vec4")) |
        value!(BasicType::Bvec2, tag!("bvec2")) |
        value!(BasicType::Bvec3, tag!("bvec3")) |
        value!(BasicType::Bvec4, tag!("bvec4")) |
        value!(BasicType::Ivec2, tag!("ivec2")) |
        value!(BasicType::Ivec3, tag!("ivec3")) |
        value!(BasicType::Ivec4, tag!("ivec4")) |
        value!(BasicType::Mat2, tag!("mat3")) |
        value!(BasicType::Mat3, tag!("mat3")) |
        value!(BasicType::Mat4, tag!("mat4")) |
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
        map!( value_text!(operator), |(s,cs)| Token::Operator(s, cs.0.into())) |
        map!( value_text!(constant), |(s,cs)| Token::Constant(s, cs.0.into())) |
        map!( value_text!(basic_type), |(s,cs)| Token::BasicType(s, cs.0.into())) |
        map!( value_text!(identifier), |(s,cs)| Token::Identifier(s, cs.0.into()))
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

    #[test]
    fn parse_valid_name() {
        let i = valid_name(CompleteStr("a"));
        assert_eq!(i, Ok((CompleteStr(""), "a".into())))
    }
}
