use uni_glsl::preprocessor;
//use uni_glsl::parser;
// use uni_glsl::TypeQualifier;
// use uni_glsl::query::*;

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
    pub filename: String,
    //unit: parser::TranslationUnit,
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
                "#version 150\n".to_string() + s
            } else {
                if s.starts_with("#define USE_GLSL_300ES") {
                    webgl::print("Use 300 es");
                    "#version 300 es\n".to_owned() + s
                } else {
                    s.to_owned()
                }
            },

            ShaderKind::Fragment => if !webgl::IS_GL_ES {
                "#version 150\n".to_string() + s
            } else {
                if s.starts_with("#define USE_GLSL_300ES") {
                    webgl::print("Use 300 es");
                    "#version 300 es\n".to_owned() + "precision highp float;\n" + s
                } else {
                    "precision highp float;\n".to_owned() + s
                }
            },
        };

        let mut predefs: HashMap<String, String> = HashMap::new();
        predefs.insert("GL_ES".to_string(), "".to_string());

        webgl::print(&format!("preprocessing {}...\n", filename));

        preprocessor::preprocess(&s, &predefs).unwrap();
        //let unit = parser::parse(&preprocessed.unwrap()).unwrap();

        Shader {
            //unit: unit,
            filename: filename.to_string(),
            code: s.to_string(),
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
