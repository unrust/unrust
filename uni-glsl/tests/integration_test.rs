extern crate uni_glsl;

use uni_glsl::preprocessor;
use uni_glsl::parser;
use uni_glsl::TypeQualifier;
use uni_glsl::BasicType;

// There are some helper query functions in this module
use uni_glsl::query::*;

use std::collections::HashMap;

#[test]
fn test_vs() {
    let test_text = include_str!("../data/test/phong_vs.glsl");

    let mut predefs = HashMap::new();

    predefs.insert("GL_ES".into(), "".into());

    let preprocessed: String = preprocessor::preprocess(test_text, &predefs, &HashMap::new()).unwrap();

    let unit = parser::parse(&preprocessed).unwrap();

    assert_eq!(unit.func_defs.len(), 1);
    let main = &unit.func_defs[0];
    assert_eq!(main.0.name, "main");

    // you can query decl by name and chain to is something.
    // currently suppoer enun:
    // TypeQualifier
    let attr_opt = unit.query_decl("aVertexPosition")
        .is(TypeQualifier::Attribute);

    assert!(attr_opt.is_some());

    // Some helper function from query trait
    let attr = attr_opt.unwrap();
    assert!(*attr.actual_type().unwrap() == BasicType::Vec3);

    // Or you can search for all
    let decls = unit.query_decl_all(TypeQualifier::Uniform);
    assert_eq!(decls.len(), 4);
}
