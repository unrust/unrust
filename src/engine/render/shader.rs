use uni_glsl::preprocessor;
use uni_glsl::preprocessor::PreprocessError;

//use uni_glsl::parser;
// use uni_glsl::TypeQualifier;
// use uni_glsl::query::*;

use webgl;
use std::collections::HashMap;
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
pub struct PreprocessedShaderCode(String);

impl PreprocessedShaderCode {
    pub fn as_string(&self) -> &String {
        &self.0
    }

    pub fn new(
        kind: ShaderKind,
        filename: &str,
        s: &str,
    ) -> Result<PreprocessedShaderCode, PreprocessError> {
        let prefix = match kind {
            ShaderKind::Vertex => if !webgl::IS_GL_ES {
                "#version 150\n".to_owned()
            } else {
                if s.starts_with("#define USE_GLSL_300ES") {
                    "#version 300 es\n".to_owned()
                } else {
                    "".to_owned()
                }
            },

            ShaderKind::Fragment => if !webgl::IS_GL_ES {
                "#version 150\n".to_owned()
            } else {
                if s.starts_with("#define USE_GLSL_300ES") {
                    "#version 300 es\n".to_owned() + "precision highp float;\n"
                } else {
                    "precision highp float;\n".to_owned()
                }
            },
        };

        let mut predefs: HashMap<String, String> = HashMap::new();
        if webgl::IS_GL_ES {
            predefs.insert("GL_ES".to_string(), "".to_string());
        }

        webgl::print(&format!("preprocessing {}...\n", filename));
        let processed = preprocessor::preprocess(&s, &predefs);

        processed.map(|s| PreprocessedShaderCode(prefix + &s))
    }
}

#[derive(Debug)]
pub struct Shader<T: ShaderKindProvider> {
    pub code: PreprocessedShaderCode,
    pub filename: String,
    //unit: parser::TranslationUnit,
    phantom: PhantomData<*const T>,
}

pub type ShaderVs = Shader<ShaderKindVs>;
pub type ShaderFs = Shader<ShaderKindFs>;

impl<T> Shader<T>
where
    T: ShaderKindProvider,
{
    pub fn new(filename: &str, s: &str) -> Shader<T> {
        let code = PreprocessedShaderCode::new(T::kind(), filename, s).unwrap();

        Shader {
            //unit: unit,
            filename: filename.to_string(),
            code,
            phantom: PhantomData,
        }
    }

    pub fn from_preprocessed(filename: &str, code: PreprocessedShaderCode) -> Shader<T> {
        Shader {
            //unit: unit,
            filename: filename.to_string(),
            code,
            phantom: PhantomData,
        }
    }

    // pub fn has_attr(&self, s: &str) -> bool {
    //     self.unit
    //         .query_decl(s)
    //         .is(TypeQualifier::Attribute)
    //         .is_some()
    // }
}
