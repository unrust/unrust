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
    Or,
    Xor,
    And,
    BitOr,
    BitXor,
    BitAnd,
    Equal,
    NonEqual,
    LT,
    GT,
    LTE,
    GTE,
    LShift,
    RShift,
    Add,
    Sub,
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
                Result::Ok( (curi1, e1) ) => { input = curi1; $e = e1; true }
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
        do_parse!(
            init_expr: unary_expression >>
            part: fold_left_alt!(e; init_expr;
                map!( preceded!(op!(Operator::Star), unary_expression),
                    |e1| Expression::Binary(BinaryOp::Mult, Box::new(e.clone()), Box::new(e1))) |
                map!( preceded!(op!(Operator::Slash), unary_expression),
                    |e1| Expression::Binary(BinaryOp::Div, Box::new(e.clone()), Box::new(e1))) |
                map!( preceded!(op!(Operator::Percent), unary_expression),
                    |e1| Expression::Binary(BinaryOp::Mod, Box::new(e.clone()), Box::new(e1)))
            ) >> (part)
        )
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
    add_expression<CS, Expression>,
    ows!(
        do_parse!(
            init_expr: mult_expression >>
            part: fold_left_alt!(e; init_expr;
                map!( preceded!(op!(Operator::Plus), mult_expression), 
                    |e1| Expression::Binary(BinaryOp::Add, Box::new(e.clone()), Box::new(e1))) |
                map!( preceded!(op!(Operator::Dash), mult_expression), 
                    |e1| Expression::Binary(BinaryOp::Sub, Box::new(e.clone()), Box::new(e1)))                
            ) >> (part)
        )
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
    shift_expression<CS, Expression>,
    ows!(
        do_parse!(
            init_expr: add_expression >>
            part: fold_left_alt!(e; init_expr;
                map!( preceded!(op!(Operator::LeftOp), add_expression), 
                    |e1| Expression::Binary(BinaryOp::LShift, Box::new(e.clone()), Box::new(e1))) |
                map!( preceded!(op!(Operator::RightOp), add_expression), 
                    |e1| Expression::Binary(BinaryOp::RShift, Box::new(e.clone()), Box::new(e1)))                
            ) >> (part)
        )
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
    relational_expression<CS, Expression>,
    ows!(
        do_parse!(
            init_expr: shift_expression >>
            part: fold_left_alt!(e; init_expr;
                map!( preceded!(op!(Operator::LeftAngle), shift_expression), 
                    |e1| Expression::Binary(BinaryOp::LT, Box::new(e.clone()), Box::new(e1))) |
                map!( preceded!(op!(Operator::RightAngle), shift_expression), 
                    |e1| Expression::Binary(BinaryOp::GT, Box::new(e.clone()), Box::new(e1))) |
                map!( preceded!(op!(Operator::LeOp), shift_expression), 
                    |e1| Expression::Binary(BinaryOp::LTE, Box::new(e.clone()), Box::new(e1))) |
                map!( preceded!(op!(Operator::GeOp), shift_expression), 
                    |e1| Expression::Binary(BinaryOp::GTE, Box::new(e.clone()), Box::new(e1)))                
            ) >> (part)
        )
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
    equality_expression<CS, Expression>,
    ows!(
        do_parse!(
            init_expr: relational_expression >>
            part: fold_left_alt!(e; init_expr;
                map!( preceded!(op!(Operator::EqOp), relational_expression), 
                    |e1| Expression::Binary(BinaryOp::Equal, Box::new(e.clone()), Box::new(e1))) |
                map!( preceded!(op!(Operator::NeOp), relational_expression), 
                    |e1| Expression::Binary(BinaryOp::NonEqual, Box::new(e.clone()), Box::new(e1))) 
            ) >> (part)
        )
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
    bit_and_expression<CS, Expression>,
    ows!(
        do_parse!(
            init_expr: equality_expression >>
            part: fold_left_alt!(e; init_expr;
                map!( preceded!(op!(Operator::Ampersand), equality_expression), 
                    |e1| Expression::Binary(BinaryOp::BitAnd, Box::new(e.clone()), Box::new(e1))) 
            ) >> (part)
        )
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
    bit_xor_expression<CS, Expression>,
    ows!(
        do_parse!(
            init_expr: bit_and_expression >>
            part: fold_left_alt!(e; init_expr;
                map!( preceded!(op!(Operator::Caret), bit_and_expression), 
                    |e1| Expression::Binary(BinaryOp::BitXor, Box::new(e.clone()), Box::new(e1))) 
            ) >> (part)
        )
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
    bit_or_expression<CS, Expression>,
    ows!(
        do_parse!(
            init_expr: bit_and_expression >>
            part: fold_left_alt!(e; init_expr;
                map!( preceded!(op!(Operator::VerticalBar), bit_xor_expression), 
                    |e1| Expression::Binary(BinaryOp::BitOr, Box::new(e.clone()), Box::new(e1))) 
            ) >> (part)
        )
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
    logical_and_expression<CS, Expression>,
    ows!(
        do_parse!(
            init_expr: bit_or_expression >>
            part: fold_left_alt!(e; init_expr;
                map!( preceded!(op!(Operator::AndOp), bit_or_expression), 
                    |e1| Expression::Binary(BinaryOp::And, Box::new(e.clone()), Box::new(e1))) 
            ) >> (part)
        )
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
    logical_xor_expression<CS, Expression>,
    ows!(
        do_parse!(
            init_expr: logical_and_expression >>
            part: fold_left_alt!(e; init_expr;
                map!( preceded!(op!(Operator::XorOp), logical_and_expression), 
                    |e1| Expression::Binary(BinaryOp::Xor, Box::new(e.clone()), Box::new(e1))) 
            ) >> (part)
        )
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
    logical_or_expression<CS, Expression>,
    ows!(
        do_parse!(
            init_expr: logical_xor_expression >>
            part: fold_left_alt!(e; init_expr;
                map!( preceded!(op!(Operator::OrOp), logical_xor_expression), 
                    |e1| Expression::Binary(BinaryOp::Or, Box::new(e.clone()), Box::new(e1))) 
            ) >> (part)
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

    #[test]
    fn parse_mult_expression() {
        let i = mult_expression(CompleteStr("i++ * j--"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(Mult, PostInc(Identifier(\"i\")), PostDec(Identifier(\"j\")))"
        );

        let i = mult_expression(CompleteStr("a*b*c"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(Mult, Binary(Mult, Identifier(\"a\"), Identifier(\"b\")), Identifier(\"c\"))"
        );
    }

    #[test]
    fn parse_add_expression() {
        let i = add_expression(CompleteStr("a*b + j - i"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(Sub, Binary(Add, Binary(Mult, Identifier(\"a\"), Identifier(\"b\")), Identifier(\"j\")), Identifier(\"i\"))"
        );
    }

    #[test]
    fn parse_shift_expression() {
        let i = shift_expression(CompleteStr("a*b >> 2"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(RShift, Binary(Mult, Identifier(\"a\"), Identifier(\"b\")), Constant(Constant::Integer { 2 }))"
        );
    }

    #[test]
    fn parse_relational_expression() {
        let i = relational_expression(CompleteStr("2<<1 >= 1 << a"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(GTE, Binary(LShift, Constant(Constant::Integer { 2 }), Constant(Constant::Integer { 1 })), Binary(LShift, Constant(Constant::Integer { 1 }), Identifier(\"a\")))"
        );

        let i = relational_expression(CompleteStr("a <= 1 << a"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(LTE, Identifier(\"a\"), Binary(LShift, Constant(Constant::Integer { 1 }), Identifier(\"a\")))"
        );

        let i = relational_expression(CompleteStr("2<<1 > b"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(GT, Binary(LShift, Constant(Constant::Integer { 2 }), Constant(Constant::Integer { 1 })), Identifier(\"b\"))"
        );

        let i = relational_expression(CompleteStr("x < y"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(LT, Identifier(\"x\"), Identifier(\"y\"))"
        );
    }

    #[test]
    fn parse_equality_expression() {
        let i = equality_expression(CompleteStr("2 == 3"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(Equal, Constant(Constant::Integer { 2 }), Constant(Constant::Integer { 3 }))"
        );

        let i = equality_expression(CompleteStr("a != a"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(NonEqual, Identifier(\"a\"), Identifier(\"a\"))"
        );
    }

    #[test]
    fn parse_bit_and_expression() {
        let i = bit_and_expression(CompleteStr("2 == 3 & 4 == 5"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(BitAnd, Binary(Equal, Constant(Constant::Integer { 2 }), Constant(Constant::Integer { 3 })), Binary(Equal, Constant(Constant::Integer { 4 }), Constant(Constant::Integer { 5 })))"
        );
    }

    #[test]
    fn parse_bit_xor_expression() {
        let i = bit_xor_expression(CompleteStr("2 & 3 ^ 4"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(BitXor, Binary(BitAnd, Constant(Constant::Integer { 2 }), Constant(Constant::Integer { 3 })), Constant(Constant::Integer { 4 }))"
        );
    }

    #[test]
    fn parse_bit_or_expression() {
        let i = bit_or_expression(CompleteStr("2 & 3 | 4"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(BitOr, Binary(BitAnd, Constant(Constant::Integer { 2 }), Constant(Constant::Integer { 3 })), Constant(Constant::Integer { 4 }))"
        );
    }

    #[test]
    fn parse_logical_and_expression() {
        let i = logical_and_expression(CompleteStr("2 && 3"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(And, Constant(Constant::Integer { 2 }), Constant(Constant::Integer { 3 }))"
        );
    }

    #[test]
    fn parse_logical_xor_expression() {
        let i = logical_xor_expression(CompleteStr("2 ^^ 3"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(Xor, Constant(Constant::Integer { 2 }), Constant(Constant::Integer { 3 }))"
        );
    }

    #[test]
    fn parse_logical_or_expression() {
        let i = logical_or_expression(CompleteStr("2 || 3"));
        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Binary(Or, Constant(Constant::Integer { 2 }), Constant(Constant::Integer { 3 }))"
        );
    }
}
