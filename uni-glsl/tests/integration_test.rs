extern crate uni_glsl;

use uni_glsl::preprocessor;
use uni_glsl::parser;

use std::collections::HashMap;

#[test]
fn test_vs() {
    let test_text = include_str!("../data/test/phong_vs.glsl");

    let mut predefs = HashMap::new();

    predefs.insert("GL_ES".into(), "".into());

    let preprocessed: String = preprocessor::preprocess(test_text, &predefs).unwrap();

    let unit = parser::parse(&preprocessed).unwrap();

    assert_eq!(unit.func_defs.len(), 1);

    let main = &unit.func_defs[0];

    assert_eq!(main.0.name, "main");
}
