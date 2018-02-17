use nom::types::CompleteStr;
use nom::{space, Err, IResult};
use nom::line_ending;
use std::convert::From;
use std::collections::HashMap;
use std::error;
use std::fmt;
use token::*;

type CS<'a> = CompleteStr<'a>;

named!(pub lines<CS, String>, map!( many0!(
    map!(line, |s|{ CompleteStr(s.0.trim_right()) })
), line_concat));

fn line_concat(input: Vec<CompleteStr>) -> String {
    input
        .into_iter()
        .fold((String::from(""), true), |(mut c, first), s| {
            if first {
                return (s.0.into(), false);
            }

            if !c.ends_with("\\") {
                (c + "\n".into() + s.0.into(), false)
            } else {
                c.pop();
                (c + s.0.into(), false)
            }
        })
        .0
}

// Parser rewriter, discarding whitespaces and comments.
macro_rules! comment_eater {
  ($i:expr, $($args:tt)*) => {{
    sep!($i, comment, $($args)*)
  }}
}

macro_rules! spe {
  ($i:expr, $($args:tt)*) => {{
    delimited!($i, opt!(space), $($args)*, opt!(space))
  }}
}

/// Parse a single comment.
named!(pub comment<CS, CS>,
    alt!(
    complete!(preceded!(tag!("//"), take_until!("\n"))) |
    complete!(delimited!(tag!("/*"), take_until!("*/"), tag!("*/"))) |
    eat_separator!(&b" \t"[..])
    )
);

named!(line<CS, CS>, do_parse!(
        content: take_while!(|c| c != '\r' && c != '\n' ) >>        
        line_ending >>
        (content)
    )
);

fn is_not_multispace(c: char) -> bool {
    c != '\r' && c != '\n' && c != ' ' && c != '\t'
}

named!(not_whitespace<CS, CS>, do_parse!(
        tt: alt!(take_while1!(is_not_multispace) | line_ending) >> 
        (tt)
    )
);

fn join_string(input: Vec<CS>) -> String {
    input.into_iter().fold("".into(), |c, s| c + " " + s.0)
}

named!(remove_comment<CS, String>, map!( many0!(comment_eater!(not_whitespace)), join_string));

#[derive(Debug)]
enum MacroSession {
    Define(Identifier, Vec<Token>),
    Undefine(Identifier),
    IfDefine(Identifier, bool, Vec<MacroSession>, Vec<MacroSession>),
    Ignored,
    Empty,
    Normal(Vec<Token>),
    EmptyLine,
}

#[cfg_attr(rustfmt, rustfmt_skip)] 
fn macro_line<'a>(input: CompleteStr<'a>, t: &str) -> IResult<CompleteStr<'a>, Identifier> {
    do_parse!(
        input,        
        spe!(char!('#')) >>         
        tag_no_case!(t) >> 
        key: spe!(identifier) >>
        (key)
    )
}

named!(define_macro<CS, MacroSession>, 
    do_parse!(
        a: call!(macro_line, "define") >>
        tts: many0!(token) >>
        (MacroSession::Define(a, tts))
    )
);

named!(undef_macro<CS, MacroSession>, 
    map!( call!(macro_line, "undef"), MacroSession::Undefine)
);

named!(
    #[allow(unused_imports)], // fix value! warning
    empty_macro<CS, MacroSession>, 
    value!(MacroSession::Empty, spe!(char!('#')))
);

named!(ignored_macro<CS, MacroSession>, 
    do_parse!(
        spe!(char!('#')) >>
        alt!(
            tag_no_case!("error") | 
            tag_no_case!("pragma") |
            tag_no_case!("extension") |
             tag_no_case!("version") |        
             tag_no_case!("line")        
        ) >>
        many0!(token) >>
        (MacroSession::Ignored)
    )
);

named!(
    #[allow(unused_imports)], // fix value! warning
    ifdef_macro_condition<CS, bool>, 
    alt!( 
        value!(true, tag_no_case!("ifdef")) |
        value!(false, tag_no_case!("ifndef"))
    )
);

named!(normal_macro<CS,MacroSession>,
    do_parse!(
        tt: many1!(token) >>
        (MacroSession::Normal(tt))
    )
);

named!(ifdef_macro<CS, MacroSession>, 
    do_parse!(
        spe!(char!('#')) >>
        b: ifdef_macro_condition >>
        key: spe!(identifier) >>        
        line_ending >>

        // if part
        part1: many0!(terminated!(parse_macro, line_ending)) >>
        
        // else part (optional)
        part2: opt!(
            do_parse!(
                spe!(char!('#')) >>
                spe!(tag_no_case!("else")) >>        
                line_ending >>                  
                tts: many0!(terminated!(parse_macro, line_ending)) >>
                (tts)
            )
        ) >>
        
        spe!(return_error!(ErrorKind::Custom(1), char!('#'))) >>
        spe!(return_error!(ErrorKind::Custom(1), tag_no_case!("endif"))) >>            
        
        (MacroSession::IfDefine(key, b, part1, part2.unwrap_or(Vec::new())))
    )
);

named!(
    #[allow(unused_imports)],  // fix value! warning
    parse_macro<CS, MacroSession>,
    alt!(
        ifdef_macro |
        ignored_macro |
        define_macro |
        undef_macro |
        empty_macro |
        normal_macro |        
        value!(MacroSession::EmptyLine,space)
    )
);

fn append_vec<T>(mut ls: Vec<T>, last: Option<T>) -> Vec<T> {
    match last {
        Some(l) => {
            ls.push(l);
            ls
        }
        None => ls,
    }
}

#[rustfmt_skip] 
named!(
    #[allow(unused_imports)],  // fix value! warning
    preprocess_parser <CS,Vec<MacroSession>>,
    do_parse!(
        tts: many0!(alt!(
            value!(MacroSession::EmptyLine, line_ending) |
            terminated!(parse_macro, line_ending)
        )) >> 
        last: opt!(parse_macro) >> 
        (append_vec(tts,last))
    )
);

#[derive(Default)]
struct PreprocessState {
    defines: HashMap<String, Vec<Token>>,
    normal_tokens: Vec<String>,
}

fn preprocess_token(tt: Token, state: &mut PreprocessState) {
    match tt {
        Token::Identifier(s, ..) => {
            let found = { state.defines.get(&s).map(|tts| tts.clone()) };
            match found {
                Some(childs) => for child in childs.into_iter() {
                    preprocess_token(child, state);
                },
                None => {
                    state.normal_tokens.push(s);
                }
            }
        }
        Token::Operator(_, s) => {
            state.normal_tokens.push(s);
        }
        Token::Constant(_, s) => {
            state.normal_tokens.push(s);
        }
        Token::BasicType(_t, s) => {
            state.normal_tokens.push(s);
        }
    }
}

fn preprocess_session(s: MacroSession, state: &mut PreprocessState) {
    match s {
        MacroSession::EmptyLine => (),
        MacroSession::Empty => (),
        MacroSession::Define(ident, values) => {
            state.defines.insert(ident, values);
        }
        MacroSession::Undefine(ident) => {
            state.defines.remove(&ident);
        }
        MacroSession::Ignored => (),
        MacroSession::IfDefine(ident, b, first, second) => {
            let contain = state.defines.contains_key(&ident);
            let doit = (contain && b) || (!contain && !b);

            if doit {
                for child in first.into_iter() {
                    preprocess_session(child, state);
                }
            } else {
                for child in second.into_iter() {
                    preprocess_session(child, state);
                }
            }
        }
        MacroSession::Normal(n) => {
            for tt in n.into_iter() {
                preprocess_token(tt, state)
            }

            state.normal_tokens.push("\n".into());
        }
    }
}

#[derive(Debug)]
pub struct PreprocessError(String);

impl error::Error for PreprocessError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PreprocessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> From<Err<CompleteStr<'a>>> for PreprocessError {
    fn from(error: Err<CompleteStr>) -> Self {
        PreprocessError(match error {
            Err::Incomplete(needed) => format!("Imcompleted : {:?}", needed),
            Err::Error(ctx) => format!("Preprocess Error {:?}", ctx),
            Err::Failure(f) => format!("Preprocess Failure {:?}", f),
        })
    }
}

/// Implemented
///
/// #
/// #define
/// #undef
///
/// #ifdef
/// #ifndef
/// #else
/// #endif
///
/// Ignored :
///
/// #line
/// #version
/// #extension
/// #pragma
///
/// Not implemted yet:
///
/// #if
/// #elif
/// defined

pub fn preprocess(s: &str) -> Result<String, PreprocessError> {
    let stage0 = lines(CompleteStr(s))?.1;
    let stage1 = remove_comment(CompleteStr(&stage0))?.1;
    let sessions = preprocess_parser(CompleteStr(&stage1));

    let sessions = sessions?.1;
    let mut state = PreprocessState::default();

    for session in sessions.into_iter() {
        preprocess_session(session, &mut state);
    }

    Ok(state
        .normal_tokens
        .into_iter()
        .fold("".into(), |s, t| s + " " + &t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preprocess_test_file() {
        let test_text = include_str!("../data/test/preprocessor_test.glsl");
        let expect_result = include_str!("../data/test/preprocessor_test_result.glsl");
        let r = preprocess(test_text).unwrap();

        // Write result to temp directory.
        // use std::fs::File;
        // use std::io::prelude::*;
        // let mut file = File::create("D:\\Temp\\preprocessor_test_result.glsl").unwrap();
        // file.write_all(&r.as_bytes()).unwrap();

        assert_eq!(r, expect_result);
    }
}
