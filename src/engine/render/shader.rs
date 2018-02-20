use uni_glsl::preprocessor;
use uni_glsl::parser;
use uni_glsl::TypeQualifier;
use uni_glsl::query::*;

use webgl;
use std::collections::HashMap;
use std::default::Default;

#[derive(Debug, PartialEq)]
pub enum ShaderKind {
    Empty,
    Vertex,
    Fragment,
}

impl Default for ShaderKind {
    fn default() -> ShaderKind {
        ShaderKind::Empty
    }
}

#[derive(Debug, Default)]
pub struct Shader {
    pub kind: ShaderKind,
    pub code: String,
    unit: parser::TranslationUnit,
}

impl Shader {
    pub fn new(kind: ShaderKind, filename: &str, s: &str) -> Shader {
        let mut predefs: HashMap<String, String> = HashMap::new();
        predefs.insert("GL_ES".to_string(), "".to_string());

        webgl::print(&format!("preprocessing {}...\n", filename));

        let preprocessed = preprocessor::preprocess(s, &predefs);
        let unit = parser::parse(&preprocessed.unwrap()).unwrap();

        Shader {
            kind: kind,
            unit: unit,
            code: s.to_string(),
        }
    }

    pub fn has_attr(&self, s: &str) -> bool {
        self.unit
            .query_decl(s)
            .is(TypeQualifier::Attribute)
            .is_some()
    }
}
