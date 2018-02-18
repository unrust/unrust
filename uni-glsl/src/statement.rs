use nom::types::CompleteStr;
use declaration::{declaration, fully_type_specifier, initializer, Declaration, FullyTypeSpecifier};
use expression::{expression, Expression};
use token::{valid_name, Identifier};

type CS<'a> = CompleteStr<'a>;

#[derive(Debug, Clone, PartialEq)]
pub enum IterationCondition {
    Expression(Expression),
    InitialVariable(FullyTypeSpecifier, Identifier, Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum JumpType {
    Continue,
    Break,
    Return,
    ReturnWith(Expression),
    Discard,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Declaration(Declaration),
    Expression(Expression),
    Selection(Expression, Box<Statement>, Option<Box<Statement>>),
    Scoped(Vec<Box<Statement>>),

    While(IterationCondition, Box<Statement>),
    DoWhile(Expression, Box<Statement>),

    /// for(Statement;IterationCondition;Expression) Statement
    For(
        Box<Statement>,
        Option<IterationCondition>,
        Option<Expression>,
        Box<Statement>,
    ),

    JumpStatment(JumpType),
}

named!(declaration_statement<CS, Statement>, 
    map!(call!(declaration), Statement::Declaration)
);

named!(
    #[allow(unused_imports)], // fix value! bug
    expression_statement<CS, Statement>,
    ows!(
        map!( alt!(
        value!(Expression::Empty, op!(Operator::SemiColon)) |
        terminated!(expression, op!(Operator::SemiColon)))
        ,Statement::Expression ))
);

// TODO: implement all statement
#[cfg_attr(rustfmt, rustfmt_skip)] 
named!(simple_statement<CS, Statement>,
    alt!(
        declaration_statement |
        expression_statement |
        selection_statement |
        iteration_statement |
        jump_statement
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)] 
named!(statement_with_scope<CS, Box<Statement>>,
    ows!(alt!(
        map!(simple_statement, |x| Box::new(x)) |
        ows!(delimited!(
            op!(Operator::LeftBrace), 
            map!(many0!(statement_with_scope), |vs| Box::new(Statement::Scoped(vs)) ),
            op!(Operator::RightBrace)
        ))
    ))
);

named!(pub statement<CS, Box<Statement>>,
    call!(statement_with_scope)
);

#[cfg_attr(rustfmt, rustfmt_skip)] 
named!( 
    selection_statement<CS,Statement>, 
    ows!(do_parse!(
        tag!("if") >> 
        op!(Operator::LeftParen) >> 
        e: expression >> 
        op!(Operator::RightParen) >> 
        first: statement_with_scope >> 
        second: opt!(
            do_parse!(
                tag!("else") >> 
                s: statement_with_scope >> (s)
            )
        ) >>         
        (Statement::Selection(e, first, second))
    ))
);
#[cfg_attr(rustfmt, rustfmt_skip)] 
named!( 
    iteration_condition<CS, IterationCondition>,
    ows!(        
        alt!( 
            do_parse!(
                ts: fully_type_specifier >>
                n: valid_name >>
                op!(Operator::Equal) >>
                init: initializer >>
                (IterationCondition::InitialVariable(ts, n, init))
            ) |            
            map!(expression, IterationCondition::Expression)
        )
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)] 
named!( 
    while_statement<CS, Statement>,
    ows!(do_parse!(
        tag!("while") >>
        op!(Operator::LeftParen) >>
        c: iteration_condition >>
        op!(Operator::RightParen) >>
        s: statement_with_scope >>
        (Statement::While(c, s))
    ))
);

#[cfg_attr(rustfmt, rustfmt_skip)] 
named!( 
    do_while_statement<CS, Statement>,
    ows!(do_parse!(
        tag!("do") >>
        s : statement_with_scope >>
        tag!("while") >>        
        op!(Operator::LeftParen) >>
        e: expression >>
        op!(Operator::RightParen) >>
        op!(Operator::SemiColon) >>
        
        (Statement::DoWhile(e, s))
    ))
);

#[cfg_attr(rustfmt, rustfmt_skip)] 
named!( 
    for_statement<CS, Statement>,
    ows!(do_parse!(
        tag!("for") >>
        op!(Operator::LeftParen) >>
        init: alt!(expression_statement | declaration_statement) >>
        rest: do_parse!(
            c: opt!(iteration_condition) >>
            op!(Operator::SemiColon) >>
            e: opt!(expression) >>
            (c,e)
        ) >>
        op!(Operator::RightParen) >>        
        s : statement_with_scope >>
        
        (Statement::For(Box::new(init), rest.0, rest.1, s))
    ))
);

#[cfg_attr(rustfmt, rustfmt_skip)] 
named!(
    #[allow(unused_imports)], // fix value! bug
    jump_statement<CS, Statement>, 
    ows!(
        do_parse!(        
            jp : alt!(
                map!(preceded!(tag!("return"), expression), JumpType::ReturnWith) |
                value!(JumpType::Continue, tag!("continue")) |
                value!(JumpType::Break, tag!("break")) |                
                value!(JumpType::Discard, tag!("discard")) |
                value!(JumpType::Return, tag!("return")) 
            ) >>
            op!(Operator::SemiColon) >>
            (Statement::JumpStatment(jp))
        )
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)] 
named!( 
    iteration_statement<CS, Statement>,
    alt!(
        while_statement |
        do_while_statement |
        for_statement
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_expression_statement() {
        let i = expression_statement(CompleteStr("a+b;"));

        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Expression(Binary(Add, Identifier(\"a\"), Identifier(\"b\")))"
        );
    }

    #[test]
    fn parse_assignment_statemant() {
        let i = expression_statement(CompleteStr("gl_FragColor=vec4(result,1.0);"));

        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Expression(Assign(Equal, Identifier(\"gl_FragColor\"), FunctionCall(Vec4, [Identifier(\"result\"), Constant(Float(1.0))])))"
        );
    }

    #[test]
    fn parse_selection_statement() {
        let i = selection_statement(CompleteStr(
            r#"
            if (x) {
                1;
                2;
            } else {
                {2;}
            }
        "#,
        ));

        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Selection(Identifier(\"x\"), Scoped([Expression(Constant(Integer(1))), Expression(Constant(Integer(2)))]), Some(Scoped([Scoped([Expression(Constant(Integer(2)))])])))"
        );

        let i = selection_statement(CompleteStr(
            r#"
            if (x) {
                1;
                2;
            }
        "#,
        ));

        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "Selection(Identifier(\"x\"), Scoped([Expression(Constant(Integer(1))), Expression(Constant(Integer(2)))]), None)"
        );
    }

    #[test]
    fn parse_while_statement() {
        let i = iteration_statement(CompleteStr(
            r#"
                while(float i = 10){
                    i++;
                }
            "#,
        ));

        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "While(InitialVariable(FullyTypeSpecifier { qualifer: None, type_spec: TypeSpecifier { precision: None, actual_type: Float } }, \"i\", Constant(Integer(10))), Scoped([Expression(PostInc(Identifier(\"i\")))]))"
        );
    }

    #[test]
    fn parse_do_while_statement() {
        let i = iteration_statement(CompleteStr(
            r#"
                do {
                    i++;
                } while(true);
            "#,
        ));

        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "DoWhile(Constant(Bool(true)), Scoped([Expression(PostInc(Identifier(\"i\")))]))"
        );
    }

    #[test]
    fn parse_for_statement() {
        let i = iteration_statement(CompleteStr(
            r#"for(int i = 0; i < 10; i++) {
                j--;
            }
            "#,
        ));

        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "For(Declaration(DeclarationList([SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: None, type_spec: TypeSpecifier { precision: None, actual_type: Int } }), name: Some(\"i\"), array_spec: None, equal_to: Some(Constant(Integer(0))) }])), Some(Expression(Binary(LT, Identifier(\"i\"), Constant(Integer(10))))), Some(PostInc(Identifier(\"i\"))), Scoped([Expression(PostDec(Identifier(\"j\")))]))"
        );
    }

    #[test]
    fn parse_jump_statement() {
        let i = simple_statement(CompleteStr("return x;"));

        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "JumpStatment(ReturnWith(Identifier(\"x\")))"
        );
    }
}
