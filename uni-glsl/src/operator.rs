use nom::types::CompleteStr;

type CS<'a> = CompleteStr<'a>;

macro_rules! op {
  ($i:expr, $target:tt :: $t2:tt ) => {{
    tag!($i, $crate::operator::internal::$t2())
  }};

}

macro_rules! operator_enum_define {
    ( $($arg:ident >> $e:expr ),* ) => {
        #[derive(Debug, Clone, PartialEq, Copy, Hash, Eq)]
        pub enum Operator {
            $($arg),*
        }

        pub mod internal {
            $(
                #[allow(non_snake_case)]
                #[inline]
                pub fn $arg() -> &'static str {
                    $e
                }
            )*
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

        //TODO: Dont use this loop all method.
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
