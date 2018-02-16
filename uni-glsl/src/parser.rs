use nom::types::CompleteStr;
use super::tokens::*;
use operators::{operator, Operator};
use nom::{sp, IResult};

type CS<'a> = CompleteStr<'a>;

// Parser rewriter, discarding optional whitespaces
named!(ospace<CS, Option<CS>>, opt!(sp));

macro_rules! ows {
  ($i:expr, $($args:tt)*) => {{
    sep!($i, ospace, $($args)*)
  }}
}

macro_rules! op {
  ($i:expr, $target:expr) => {{
    verify!($i, operator, move |c| c == $target)
  }}
}

#[derive(Clone, PartialEq, Debug)]
enum BinaryOp {
    // Or,
    // Xor,
    // And,
    // BitOr,
    // BitXor,
    // BitAnd,
    // Equal,
    // NonEqual,
    // LT,
    // GT,
    // LTE,
    // GTE,
    // LShift,
    // RShift,
    // Add,
    //Sub,
    Mult,
    Div,
    Mod,
}

#[derive(Clone, PartialEq, Debug)]
enum Expression {
    Identifier(Identifier),
    Constant(Constant),
    Bracket(Box<Expression>, Box<Expression>),
    FunctionCall(BasicType, Vec<Expression>),
    DotField(Box<Expression>, Identifier),

    PostInc(Box<Expression>),
    PostDec(Box<Expression>),

    PreInc(Box<Expression>),
    PreDec(Box<Expression>),

    Plus(Box<Expression>),
    Minus(Box<Expression>),
    Not(Box<Expression>),
    Tilde(Box<Expression>),

    Binary(BinaryOp, Box<Expression>, Box<Expression>),
}

named!(expression<CS, Expression>, 
    call!(postfix_expression)
);

named!(assignment_expression<CS, Expression>, 
    call!(postfix_expression)
);

named!(
    function_call<CS, Expression>,
    do_parse!(
        callee : alt!( 
            basic_type |  map!(identifier, BasicType::TypeName)
        ) >>
        op!(Operator::LeftParen) >>
        es: alt!(
            value!(Vec::new(), tag!("void")) | 
            separated_list!(op!(Operator::Comma), assignment_expression)
        ) >>
        op!(Operator::RightParen) >>
        (Expression::FunctionCall(callee, es))
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)] 
named!(array_specifier<CS, Expression> ,
    ows!( delimited!(op!(Operator::LeftBracket), expression, op!(Operator::RightBracket)) )
);

#[cfg_attr(rustfmt, rustfmt_skip)] 
named!(dot_field_specifier<CS, Identifier>,
    ows!( preceded!(op!(Operator::Dot), identifier) )
);

macro_rules! fold_left_alt {
    ($i:expr, $e:ident; $init_expr:expr; $($args:tt)*) => {{
        let mut found = true;
        let mut $e = $init_expr;
        let mut input = $i;

        while found {
            #![allow(unused_imports)]
            let r = alt!(input, $($args)*);

            found = match r {
                Result::Ok( (i1, e1) ) => { input = i1; $e = e1; true }
                Result::Err(_) => false,
            };
        }

        Result::Ok( (input, $e.clone()) )
    }}
}

#[cfg_attr(rustfmt, rustfmt_skip)] 
named!(
    postfix_expression<CS,Expression>,
    ows!(do_parse!(
        init_expr: alt!(
            function_call |
            delimited!(op!(Operator::LeftParen), expression, op!(Operator::RightParen)) |
            map!(identifier, |i| Expression::Identifier(i)) | 
            map!(constant,|i| Expression::Constant(i) ) 
            ) >> 
        part: fold_left_alt!(e; init_expr;
            map!(array_specifier, |r| { Expression::Bracket(Box::new(e.clone()), Box::new(r)) })  |
            map!(dot_field_specifier, |r| { Expression::DotField(Box::new(e.clone()), r) })  |
            value!(Expression::PostInc(Box::new(e.clone())), op!(Operator::IncOp)) |
            value!(Expression::PostDec(Box::new(e.clone())), op!(Operator::DecOp))
        )
        >> (part)
    ))
);

#[cfg_attr(rustfmt, rustfmt_skip)] 
fn postfix_expression_part<'a>(mut input: CompleteStr<'a>, mut e: Expression) -> IResult<CompleteStr<'a>, Expression> {
    #![allow(unused_imports)]
    
    let mut found = true;

    while found {
        let r = alt!(input,
            map!(array_specifier, |r| { Expression::Bracket(Box::new(e.clone()), Box::new(r)) })  |
            map!(dot_field_specifier, |r| { Expression::DotField(Box::new(e.clone()), r) })  |
            value!(Expression::PostInc(Box::new(e.clone())), op!(Operator::IncOp)) |
            value!(Expression::PostDec(Box::new(e.clone())), op!(Operator::DecOp))
        );

        found = match r {    
            Result::Ok( (i1, e1) ) => { input = i1; e = e1; true }
            Result::Err(_) => false,
        };
    }

    Result::Ok( (input, e) )
}

// #[cfg_attr(rustfmt, rustfmt_skip)]
// fn postfix_expression_part<'a>(input: CompleteStr<'a>, e: Expression) -> IResult<CompleteStr<'a>, Expression> {
//     #![allow(unused_imports)]

//     let r = alt!(input,
//         map!(array_specifier, |r| { Expression::Bracket(Box::new(e.clone()), Box::new(r)) })  |
//         map!(dot_field_specifier, |r| { Expression::DotField(Box::new(e.clone()), r) })  |
//         value!(Expression::PostInc(Box::new(e.clone())), op!(Operator::IncOp)) |
//         value!(Expression::PostDec(Box::new(e.clone())), op!(Operator::DecOp))
//     );

//     match r {
//         Result::Ok( (i1, e1) ) => postfix_expression_part(i1, e1),
//         Result::Err(_) => Result::Ok( (input, e) ),
//     }
// }

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
    unary_expression<CS,Expression>,
    ows!(
        alt!( 
            postfix_expression |
            map!( preceded!(op!(Operator::IncOp), unary_expression), |e| Expression::PreInc(Box::new(e))) |
            map!( preceded!(op!(Operator::DecOp), unary_expression), |e| Expression::PreDec(Box::new(e))) |
            map!( preceded!(op!(Operator::Plus), unary_expression), |e| Expression::Plus(Box::new(e))) |
            map!( preceded!(op!(Operator::Dash), unary_expression), |e| Expression::Minus(Box::new(e))) |
            map!( preceded!(op!(Operator::Tilde), unary_expression), |e| Expression::Tilde(Box::new(e))) 
        )
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
    mult_expression<CS, Expression>,
    ows!(
        alt!( 
            unary_expression | 
            map!( separated_pair!(mult_expression, op!(Operator::Star), mult_expression), 
                |(e0,e1)| Expression::Binary(BinaryOp::Mult, Box::new(e0), Box::new(e1))) |
            map!( separated_pair!(mult_expression, op!(Operator::Slash), mult_expression), 
                |(e0,e1)| Expression::Binary(BinaryOp::Div, Box::new(e0), Box::new(e1))) |
            map!( separated_pair!(mult_expression, op!(Operator::Percent), mult_expression), 
                |(e0,e1)| Expression::Binary(BinaryOp::Mod, Box::new(e0), Box::new(e1)))          
            
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_expression() {
        let i = postfix_expression(CompleteStr("(i ++) -- "));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            r#"PostDec(PostInc(Identifier("i")))"#
        );

        let i = postfix_expression(CompleteStr("a[b]"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            r#"Bracket(Identifier("a"), Identifier("b"))"#
        );

        let i = postfix_expression(CompleteStr("a(b,c,d,i++)"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),            
            "FunctionCall(TypeName(\"a\"), [Identifier(\"b\"), Identifier(\"c\"), Identifier(\"d\"), PostInc(Identifier(\"i\"))])"
        );
    }

    #[test]
    fn parse_unary_expression() {
        let i = unary_expression(CompleteStr("-a(b,c,d,i++)"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),            
            "Minus(FunctionCall(TypeName(\"a\"), [Identifier(\"b\"), Identifier(\"c\"), Identifier(\"d\"), PostInc(Identifier(\"i\"))]))"
        );
    }

    // #[test]
    // fn parse_mult_expression() {
    //     let i = mult_expression(CompleteStr("i++ * j--"));
    //     assert_eq!(
    //         format!("{:?}", i.unwrap().1),
    //         "Minus(FunctionCall(TypeName(\"a\"), [Identifier(\"b\"), Identifier(\"c\"), Identifier(\"d\"), PostInc(Identifier(\"i\"))]))"
    //     );
    // }
}
