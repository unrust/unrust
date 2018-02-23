use uni_glsl::preprocessor;
use uni_glsl::parser;
use uni_glsl::TypeQualifier;
use uni_glsl::query::*;

use webgl;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum ShaderKind {
    Vertex,
    Fragment,
}

#[derive(Debug)]
pub struct Shader {
    pub kind: ShaderKind,
    pub code: String,
    unit: parser::TranslationUnit,
}

impl Shader {
    pub fn new(kind: ShaderKind, filename: &str, s: &str) -> Shader {
        let s = match kind {
            ShaderKind::Vertex => if !webgl::IS_GL_ES {
                "#version 130\n".to_string() + s
            } else {
                s.to_string()
            },

            ShaderKind::Fragment => if !webgl::IS_GL_ES {
                "#version 130\n".to_string() + s
            } else {
                ("precision highp float;\n").to_string() + s
            },
        };

        let mut predefs: HashMap<String, String> = HashMap::new();
        predefs.insert("GL_ES".to_string(), "".to_string());

        webgl::print(&format!("preprocessing {}...\n", filename));

        let preprocessed = preprocessor::preprocess(&s, &predefs);
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
