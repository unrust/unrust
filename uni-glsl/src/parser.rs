use nom::types::CompleteStr;
use nom::Err;
use statement::{statement, Statement};
use declaration::{declaration, function_prototype, Declaration, FunctionPrototype};

use std::error;
use std::fmt;

type CS<'a> = CompleteStr<'a>;

#[derive(Debug, Clone, PartialEq)]
pub enum ExternalDeclaration {
    FuntionDefinition(FunctionPrototype, Box<Statement>),
    Declaration(Declaration),
}

named!(external_declaration<CS, ExternalDeclaration >, 
    ows!(alt!(
        map!(declaration, ExternalDeclaration::Declaration)|
        map!(pair!(function_prototype, statement), |(f,s)| ExternalDeclaration::FuntionDefinition(f,s))
    ))
);

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TranslationUnit {
    pub func_defs: Vec<(FunctionPrototype, Box<Statement>)>,
    pub decls: Vec<Declaration>,
}

#[cfg_attr(rustfmt, rustfmt_skip)] 
named!(
    translation_unit<CS,TranslationUnit>, 
    exact!(fold_many0!(external_declaration,TranslationUnit::default(), |mut unit: TranslationUnit, item| {
            match item {
                ExternalDeclaration::FuntionDefinition(f, s) => unit.func_defs.push((f, s)),
                ExternalDeclaration::Declaration(d) => unit.decls.push(d),
            }
            unit
        }
    ))
);

#[derive(Debug)]
pub struct ParserError(String);

impl error::Error for ParserError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> From<Err<CompleteStr<'a>>> for ParserError {
    fn from(error: Err<CompleteStr>) -> Self {
        ParserError(match error {
            Err::Incomplete(needed) => format!("Imcompleted : {:?}", needed),
            Err::Error(ctx) => format!("Parser Error {:?}", ctx),
            Err::Failure(f) => format!("Parser Failure {:?}", f),
        })
    }
}

pub fn parse(s: &str) -> Result<TranslationUnit, ParserError> {
    Ok(translation_unit(CompleteStr(s))?.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test_file() {
        let test_text = include_str!("../data/test/parser_test.glsl");
        //let expect_result = include_str!("../data/test/preprocessor_test_result.glsl");
        let r = translation_unit(CompleteStr(test_text));

        //Write result to temp directory.
        // use std::fs::File;
        // use std::io::prelude::*;
        // let mut file = File::create("D:\\Temp\\preprocessor_test_result.glsl").unwrap();
        // file.write_all(&r.as_bytes()).unwrap();

        //assert_eq!(r, expect_result);
        if r.is_err() {
            panic!(println!("{:?}", r));
        }

        let result = r.unwrap();
        if (result.0).0.len() > 0 {
            panic!(println!("{:?}", result));
        }
    }

    #[test]
    fn parse_basic_decl() {
        let i = translation_unit(CompleteStr(
            r#" struct DirectionalLight { 
 vec3 direction ; 
 vec3 ambient ; 
 vec3 diffuse ; 
 vec3 specular ; 
 }; 
"#,
        ));

        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "TranslationUnit { func_defs: [], decls: [DeclarationList([SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: None, type_spec: TypeSpecifier { precision: None, actual_type: Struct(Struct { name: Some(\"DirectionalLight\"), members: [StructMember { ts: TypeSpecifier { precision: None, actual_type: Vec3 }, name: \"direction\", array_spec: None }, StructMember { ts: TypeSpecifier { precision: None, actual_type: Vec3 }, name: \"ambient\", array_spec: None }, StructMember { ts: TypeSpecifier { precision: None, actual_type: Vec3 }, name: \"diffuse\", array_spec: None }, StructMember { ts: TypeSpecifier { precision: None, actual_type: Vec3 }, name: \"specular\", array_spec: None }] }) } }), name: None, array_spec: None, equal_to: None }])] }"
        );
    }

    #[test]
    fn parse_basic_funtion() {
        let i = translation_unit(CompleteStr(
            r#"
        void main ( void ) { 
 vec3 norm = normalize ( vNormal ) ; 
 vec3 viewDir = normalize ( uViewPos - vFragPos ) ; 
 vec3 result = CalcDirectionalLight ( uDirectionalLight , norm , viewDir ) ; 
 
 gl_FragColor = vec4 ( result , 1.0 ) ; 
 } "#,
        ));

        let result = i.unwrap();

        if (result.0).0.len() > 0 {
            panic!(result);
        }

        assert_eq!(
            format!("{:?}", result.1),
            "TranslationUnit { func_defs: [(FunctionPrototype { ret_type: FullyTypeSpecifier { qualifer: None, type_spec: TypeSpecifier { precision: None, actual_type: Void } }, name: \"main\", params: [ParamDeclaration { type_qualifer: None, param_qualifier: None, type_spec: TypeSpecifier { precision: None, actual_type: Void }, name: None, array_spec: None }] }, Scoped([Declaration(DeclarationList([SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: None, type_spec: TypeSpecifier { precision: None, actual_type: Vec3 } }), name: Some(\"norm\"), array_spec: None, equal_to: Some(FunctionCall(TypeName(\"normalize\"), [Identifier(\"vNormal\")])) }])), Declaration(DeclarationList([SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: None, type_spec: TypeSpecifier { precision: None, actual_type: Vec3 } }), name: Some(\"viewDir\"), array_spec: None, equal_to: Some(FunctionCall(TypeName(\"normalize\"), [Binary(Sub, Identifier(\"uViewPos\"), Identifier(\"vFragPos\"))])) }])), Declaration(DeclarationList([SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: None, type_spec: TypeSpecifier { precision: None, actual_type: Vec3 } }), name: Some(\"result\"), array_spec: None, equal_to: Some(FunctionCall(TypeName(\"CalcDirectionalLight\"), [Identifier(\"uDirectionalLight\"), Identifier(\"norm\"), Identifier(\"viewDir\")])) }])), Expression(Assign(Equal, Identifier(\"gl_FragColor\"), FunctionCall(Vec4, [Identifier(\"result\"), Constant(Float(1.0))])))]))], decls: [] }"
        );
    }
}
