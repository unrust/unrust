use uni_glsl::preprocessor;
use uni_glsl::parser;
use uni_glsl::{Declaration, SingleDeclaration, TypeQualifier, VariantTypeSpecifier};
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

    pub fn has_input(&self, s: &str) -> bool {
        if let Some(ref sdec) = self.get_decl(s) {
            if let VariantTypeSpecifier::Normal(ref full_ts) = sdec.type_spec {
                if let Some(ref tq) = full_ts.qualifer {
                    return match tq {
                        &TypeQualifier::Attribute => true,
                        &TypeQualifier::Varying => self.kind == ShaderKind::Fragment,
                        _ => false,
                    };
                }
            }
        }

        false
    }

    pub fn get_decl(&self, s: &str) -> Option<&SingleDeclaration> {
        for decl in self.unit.decls.iter() {
            if let &Declaration::DeclarationList(ref list) = decl {
                for sdecl in list.iter() {
                    if let Some(ref name) = sdecl.name {
                        if name == s {
                            return Some(sdecl);
                        }
                    }
                }
            }
        }

        None
    }
}
