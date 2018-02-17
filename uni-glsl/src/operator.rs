use nom::types::CompleteStr;

type CS<'a> = CompleteStr<'a>;

macro_rules! op {
  ($i:expr, $target:expr) => {{
    verify!($i, $crate::operator::operator, move |c| c == $target)
  }}
}

macro_rules! operator_enum_define {
    ( $($arg:ident >> $e:expr ),* ) => {
        #[derive(Debug, Clone, PartialEq, Copy)]
        pub enum Operator {
            $($arg),*
        }

        impl Operator {
            fn from(s :&str) -> Operator {
                match s {
                    $(
                        $e => Operator::$arg
                    ),*,
                    _ => unreachable!(),
                }
            }
        }

        /// operator macro
        named!(pub operator<CS,Operator>,
            map!(alt!(
                $( tag!($e) ) |*
            ), |cs| Operator::from(cs.0))
        );
    };
}

operator_enum_define! {
    LeftOp       >> "<<",
    RightOp      >> ">>",
    IncOp        >> "++",
    DecOp        >> "--",
    LeOp         >> "<=",
    GeOp         >> ">=",
    EqOp         >> "==",
    NeOp         >> "!=",
    AndOp        >> "&&",
    OrOp         >> "||",
    XorOp        >> "^^",
    MulAssign    >> "*=",
    DivAssign    >> "/=",
    AddAssign    >> "+=",
    ModAssign    >> "%=",
    LeftAssign   >> "<<=",
    RightAssign  >> ">>=",
    AndAssign    >> "&=",
    XorAssign    >> "^=",
    OrAssign     >> "|=",
    SubAssign    >> "-=",

    LeftParen    >> "(",
    RightParen   >> ")",
    LeftBracket  >> "[",
    RightBracket >> "]",
    LeftBrace    >> "{",
    RightBrace   >> "}",
    Dot          >> ".",
    Comma        >> ",",
    Colon        >> ":",
    Equal        >> "=",
    SemiColon    >> ";",
    Bang         >> "!",
    Dash         >> "-",
    Tilde        >> "~",
    Plus         >> "+",
    Star         >> "*",
    Slash        >> "/",
    Percent      >> "%",
    LeftAngle    >> "<",
    RightAngle   >> ">",
    VerticalBar  >> "|",
    Caret        >> "^",
    Ampersand    >> "&",
    Question     >> "?"
}
