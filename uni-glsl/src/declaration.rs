use nom::types::CompleteStr;
use token::{basic_type, valid_name, BasicType, Identifier};
use operator::Operator;
use expression::{array_expression_specifier, assignment_expression, Expression};
use nom::IResult;

type CS<'a> = CompleteStr<'a>;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct FunctionPrototype {
    pub ret_type: FullyTypeSpecifier,
    pub name: Identifier,
    pub params: Vec<ParamDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum TypeQualifier {
    Const,
    Attribute,
    Varying,
    InvariantVarying,
    Uniform,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum PrecisionQualifier {
    High,
    Medium,
    Low,
}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct StructMember {
    pub ts: TypeSpecifier,
    pub name: Identifier,
    pub array_spec: Option<Expression>,
}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct Struct {
    pub name: Option<Identifier>,
    pub members: Vec<StructMember>,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct TypeSpecifier {
    pub precision: Option<PrecisionQualifier>,
    pub actual_type: BasicType,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct FullyTypeSpecifier {
    pub qualifer: Option<TypeQualifier>,
    pub type_spec: TypeSpecifier,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum ParamQualifier {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct ParamDeclaration {
    pub type_qualifer: Option<TypeQualifier>,
    pub param_qualifier: Option<ParamQualifier>,
    pub type_spec: TypeSpecifier,
    pub name: Option<Identifier>,
    pub array_spec: Option<Expression>,
}

named!(
    #[allow(unused_imports)], // fix value! bug
    param_qualifier<CS, ParamQualifier>,
    alt!(
        value!(ParamQualifier::InOut, tag!("inout")) |
        value!(ParamQualifier::In, tag!("in")) |
        value!(ParamQualifier::Out, tag!("out"))
    )
);

named!(param_declaration<CS, ParamDeclaration>,
    ows!(do_parse!(
        tq: opt!(type_qualifier) >>
        pq: opt!(param_qualifier) >>
        ts: type_specifier >>
        n:  opt!(valid_name) >>
        a:  opt!(array_expression_specifier) >>
        (ParamDeclaration{
            type_qualifer : tq,
            param_qualifier: pq,
            type_spec: ts,
            name: n,
            array_spec: a
        })
    ))
);

named!(
    #[allow(unused_imports)], // fix value! bug
    precision_qualifier<CS, PrecisionQualifier>,
    alt!(
        value!(PrecisionQualifier::High, tag!("highp")) |
        value!(PrecisionQualifier::Medium, tag!("mediump")) |
        value!(PrecisionQualifier::Low, tag!("lowp"))
    )
);

named!(
    struct_member_declaration<CS, Vec<StructMember>>,
    ows!(do_parse!(
        ts: type_specifier >>
        members: separated_nonempty_list!(
            op!(Operator::Comma),
            do_parse!(
                name: valid_name >>
                ar: opt!(array_expression_specifier) >>
                (StructMember{
                    ts: ts.clone(),
                    name: name,
                    array_spec: ar,
                })                
            )
        ) >>
        op!(Operator::SemiColon) >>
        (members)
    ))        
);

named!(
    struct_specifier<CS, BasicType>,
    ows!(do_parse!(
         tag!("struct") >>
         n: opt!(valid_name) >>
         op!(Operator::LeftBrace) >>
         ls_members: many0!(struct_member_declaration) >>
         op!(Operator::RightBrace) >>
         (
             BasicType::Struct(Struct {
                 name: n,
                 members: ls_members.into_iter().flat_map(|vm| vm.into_iter() ).collect()
             })
         )
    ))
);

named!(  
    type_specifier<CS, TypeSpecifier>,     
    ows!(do_parse!(
        p: opt!(precision_qualifier) >>
        t: alt!(basic_type | struct_specifier | map!(valid_name, BasicType::TypeName)) >>
        (TypeSpecifier {
            precision : p,
            actual_type : t
        })
    ))
);

named!(
    #[allow(unused_imports)], // fix value! bug
    type_qualifier<CS, TypeQualifier>,
    alt!(
        value!(TypeQualifier::Const, tag!("const")) |
        value!(TypeQualifier::Attribute, tag!("attribute")) |
        value!(TypeQualifier::InvariantVarying, pair!( tag!("invariant"),tag!("varying"))) |
        value!(TypeQualifier::Varying, tag!("varying")) |        
        value!(TypeQualifier::Uniform, tag!("uniform"))
    )
);

named!(
    pub fully_type_specifier<CS, FullyTypeSpecifier>,     
    ows!(do_parse!(
        q: opt!(type_qualifier) >>
        ts: type_specifier >>
        (FullyTypeSpecifier {
            qualifer : q,
            type_spec : ts
        })
    ))
);

named!(pub function_prototype<CS, FunctionPrototype>, 
    ows!(do_parse!(
        ts : fully_type_specifier >>
        ident: valid_name >>
        op!(Operator::LeftParen) >>
        params: ows!(separated_list!(op!(Operator::Comma), param_declaration)) >>
        op!(Operator::RightParen) >>        

        (FunctionPrototype {
            ret_type: ts,
            name: ident,
            params: params
        })
    ))
);

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum VariantTypeSpecifier {
    Normal(FullyTypeSpecifier),
    Invariant,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct SingleDeclaration {
    type_spec: VariantTypeSpecifier,
    name: Option<Identifier>,
    array_spec: Option<Expression>,
    equal_to: Option<Expression>,
}

named!(pub initializer<CS, Expression>,
    call!(assignment_expression)
);

named!(
    #[allow(unused_imports)], // fix value! bug
    single_declaration<CS,SingleDeclaration>,
    ows!(alt!(
        do_parse!(
            ts: value!(VariantTypeSpecifier::Invariant, tag!("invariant")) >>
            n: valid_name >> 
            (SingleDeclaration{
                type_spec: ts,
                name: Some(n),
                array_spec: None,
                equal_to: None,
            })
        ) | 
        do_parse!(
            ts : map!(fully_type_specifier, VariantTypeSpecifier::Normal) >> 
            n : valid_name >>
            eq : preceded!(op!(Operator::Equal), initializer) >>
            (SingleDeclaration{
                type_spec: ts,
                name: Some(n),
                array_spec: None,
                equal_to: Some(eq),
            })            
        ) |
        do_parse!(
            ts : map!(fully_type_specifier, VariantTypeSpecifier::Normal) >> 
            n : opt!(valid_name) >>
            a : opt!(array_expression_specifier) >> 
            (SingleDeclaration{
                type_spec: ts,
                name: n,
                array_spec: a,
                equal_to: None,
            })            
        )
    ))    
);

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Declaration {
    FunctionPrototype(FunctionPrototype),
    DeclarationList(Vec<SingleDeclaration>),
    Precision(PrecisionQualifier, BasicType),
}

#[cfg_attr(rustfmt, rustfmt_skip)] 
fn declaration_list_part<'a>(input: CompleteStr<'a>, sd: &SingleDeclaration) -> IResult<CompleteStr<'a>, SingleDeclaration> {
    ows!(input, preceded!(        
        op!(Operator::Comma),
        alt!(
            do_parse!(
                n: valid_name >> 
                a: array_expression_specifier >> 
                (SingleDeclaration {
                    type_spec: sd.type_spec.clone(),
                    name: Some(n),
                    array_spec: Some(a),
                    equal_to: None,
                })
            ) 
            | do_parse!(
                n: valid_name >> 
                eq: preceded!(op!(Operator::Equal), initializer) >> 
                (SingleDeclaration {
                    type_spec: sd.type_spec.clone(),
                    name: Some(n),
                    array_spec: None,
                    equal_to: Some(eq),
                })
            )
            | do_parse!(
                n: valid_name >>                
                (SingleDeclaration {
                    type_spec: sd.type_spec.clone(),
                    name: Some(n),
                    array_spec: None,
                    equal_to: None,
                })
            )
        )
    ))
}

#[cfg_attr(rustfmt, rustfmt_skip)] 
named!(declaration_list<CS,Vec<SingleDeclaration>>,
    do_parse!(
        sd: single_declaration >>
        ls: many0!(call!(declaration_list_part, &sd))  >>         
        ({
            let mut r = ls.clone();
            r.insert(0, sd);
            r
        })
    )
);

named!(pub declaration<CS, Declaration>,
    ows!( terminated!(alt!(
        map!(function_prototype, Declaration::FunctionPrototype) |
        map!(declaration_list, Declaration::DeclarationList) |
        do_parse!(
            tag!("precision") >>
            pq: precision_qualifier >>
            ts: basic_type >>
            (Declaration::Precision(pq, ts))
        )        
    ), op!(Operator::SemiColon) ))
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_function_prototype_no_params() {
        let i = function_prototype(CompleteStr("const highp vec3 f()"));

        assert_eq!(format!("{:?}", 
            i.unwrap().1), 
            "FunctionPrototype { ret_type: FullyTypeSpecifier { qualifer: Some(Const), type_spec: TypeSpecifier { precision: Some(High), actual_type: Vec3 } }, name: \"f\", params: [] }"
            );
    }

    #[test]
    fn parse_param_decl() {
        let i = param_declaration(CompleteStr("vec3 a)"));

        assert_eq!(format!("{:?}", 
            i.unwrap().1), 
            "ParamDeclaration { type_qualifer: None, param_qualifier: None, type_spec: TypeSpecifier { precision: None, actual_type: Vec3 }, name: Some(\"a\"), array_spec: None }"
            );
    }

    #[test]
    fn parse_function_prototype_with_params() {
        let i = function_prototype(CompleteStr(
            "const highp vec3 f(const vec3 a, in Obj b , float a[2] )",
        ));

        assert_eq!(format!("{:?}",
            i.unwrap().1),
            "FunctionPrototype { ret_type: FullyTypeSpecifier { qualifer: Some(Const), type_spec: TypeSpecifier { precision: Some(High), actual_type: Vec3 } }, name: \"f\", params: [ParamDeclaration { type_qualifer: Some(Const), param_qualifier: None, type_spec: TypeSpecifier { precision: None, actual_type: Vec3 }, name: Some(\"a\"), array_spec: None }, ParamDeclaration { type_qualifer: None, param_qualifier: Some(In), type_spec: TypeSpecifier { precision: None, actual_type: TypeName(\"Obj\") }, name: Some(\"b\"), array_spec: None }, ParamDeclaration { type_qualifer: None, param_qualifier: None, type_spec: TypeSpecifier { precision: None, actual_type: Float }, name: Some(\"a\"), array_spec: Some(Constant(Integer(2))) }] }"
            );
    }

    #[test]
    fn parse_single_declaration() {
        let i = single_declaration(CompleteStr("const highp vec3 name"));
        assert_eq!(format!("{:?}",
            i.unwrap().1),
            "SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: Some(Const), type_spec: TypeSpecifier { precision: Some(High), actual_type: Vec3 } }), name: Some(\"name\"), array_spec: None, equal_to: None }"
            );

        let i = single_declaration(CompleteStr("vec3 name[12]"));
        assert_eq!(format!("{:?}",
            i.unwrap().1),
            "SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: None, type_spec: TypeSpecifier { precision: None, actual_type: Vec3 } }), name: Some(\"name\"), array_spec: Some(Constant(Integer(12))), equal_to: None }"
            );

        let i = single_declaration(CompleteStr("float name = 10"));
        assert_eq!(format!("{:?}", 
            i.unwrap().1), 
            "SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: None, type_spec: TypeSpecifier { precision: None, actual_type: Float } }), name: Some(\"name\"), array_spec: None, equal_to: Some(Constant(Integer(10))) }"
            );
    }

    #[test]
    fn parse_declaration() {
        let i = declaration(CompleteStr("const highp vec3 name;"));
        assert_eq!(format!("{:?}",
            i.unwrap().1),
            "DeclarationList([SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: Some(Const), type_spec: TypeSpecifier { precision: Some(High), actual_type: Vec3 } }), name: Some(\"name\"), array_spec: None, equal_to: None }])"
            );

        let i = declaration(CompleteStr("const highp vec3 a, b;"));
        assert_eq!(format!("{:?}",
            i.unwrap().1),
            "DeclarationList([SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: Some(Const), type_spec: TypeSpecifier { precision: Some(High), actual_type: Vec3 } }), name: Some(\"a\"), array_spec: None, equal_to: None }, SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: Some(Const), type_spec: TypeSpecifier { precision: Some(High), actual_type: Vec3 } }), name: Some(\"b\"), array_spec: None, equal_to: None }])"
            );

        let i = declaration(CompleteStr("const highp vec3 name ;"));
        assert_eq!(format!("{:?}",
            i.unwrap().1),
            "DeclarationList([SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: Some(Const), type_spec: TypeSpecifier { precision: Some(High), actual_type: Vec3 } }), name: Some(\"name\"), array_spec: None, equal_to: None }])"
            );
    }

    #[test]
    fn parse_struct_declaration() {
        let i = declaration(CompleteStr("struct A { vec3 x,y,z; float f; };"));

        assert_eq!(format!("{:?}", i.unwrap().1), "DeclarationList([SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: None, type_spec: TypeSpecifier { precision: None, actual_type: Struct(Struct { name: Some(\"A\"), members: [StructMember { ts: TypeSpecifier { precision: None, actual_type: Vec3 }, name: \"x\", array_spec: None }, StructMember { ts: TypeSpecifier { precision: None, actual_type: Vec3 }, name: \"y\", array_spec: None }, StructMember { ts: TypeSpecifier { precision: None, actual_type: Vec3 }, name: \"z\", array_spec: None }, StructMember { ts: TypeSpecifier { precision: None, actual_type: Float }, name: \"f\", array_spec: None }] }) } }), name: None, array_spec: None, equal_to: None }])");
    }

}
