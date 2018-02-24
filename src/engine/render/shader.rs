use uni_glsl::preprocessor;
use uni_glsl::parser;
use uni_glsl::TypeQualifier;
use uni_glsl::query::*;

use webgl;
use std::collections::HashMap;
use engine::asset::loader::Loadable;
use engine::asset::loader;
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub enum ShaderKind {
    Vertex,
    Fragment,
}

pub trait ShaderKindProvider {
    fn kind() -> ShaderKind;
}

#[derive(Debug)]
pub struct ShaderKindVs {}
impl ShaderKindProvider for ShaderKindVs {
    fn kind() -> ShaderKind {
        ShaderKind::Vertex
    }
}

#[derive(Debug)]
pub struct ShaderKindFs {}
impl ShaderKindProvider for ShaderKindFs {
    fn kind() -> ShaderKind {
        ShaderKind::Fragment
    }
}

#[derive(Debug)]
pub struct Shader<T: ShaderKindProvider> {
    pub code: String,
    unit: parser::TranslationUnit,
    phantom: PhantomData<*const T>,
}

impl Loadable for Shader<ShaderKindVs> {
    type Loader = loader::ShaderVSLoader;
}

impl Loadable for Shader<ShaderKindFs> {
    type Loader = loader::ShaderFSLoader;
}

pub type ShaderVs = Shader<ShaderKindVs>;
pub type ShaderFs = Shader<ShaderKindFs>;

impl<T> Shader<T>
where
    T: ShaderKindProvider,
{
    pub fn new(filename: &str, s: &str) -> Shader<T> {
        let s = match T::kind() {
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
            unit: unit,
            code: s.to_string(),
            phantom: PhantomData,
        }
    }

    pub fn has_attr(&self, s: &str) -> bool {
        self.unit
            .query_decl(s)
            .is(TypeQualifier::Attribute)
            .is_some()
    }
}
