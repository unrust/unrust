use nom::types::CompleteStr;
use nom::{space, Err, IResult};
use nom::line_ending;
use std::convert::From;
use std::collections::HashMap;
use std::error;
use std::fmt;
use token::{identifier, token, BasicType, Constant, Identifier, Token};
use operator::Operator;
use defeval::{Eval, EvalContext, EvalError};

use expression::{expression, Expression};

type CS<'a> = CompleteStr<'a>;

fn append_vec<T>(mut ls: Vec<T>, last: Option<T>) -> Vec<T> {
    match last {
        Some(l) => {
            ls.push(l);
            ls
        }
        None => ls,
    }
}

named!(single_line<CS, CS>,
    take_while!(|c| c != '\r' && c != '\n' )
);

named!(pub lines<CS, String>, map!( 
    do_parse!(
        ls: many0!( terminated!(single_line, line_ending) ) >>
        last: opt!(single_line) >>
        (append_vec(ls, last))    
    )
, line_concat));

fn line_concat(input: Vec<CompleteStr>) -> String {
    input
        .into_iter()
        .fold((String::from(""), true), |(mut c, first), s| {
            let ts = s.0.trim_right().into();

            if first {
                return (ts, false);
            }

            if !c.ends_with("\\") {
                (c + "\n".into() + &ts, false)
            } else {
                c.pop();
                (c + &ts, false)
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
named!(#[allow(unused_imports)], // fix value! warning
    pub comment<CS, CS>,
    alt!(
    complete!(preceded!(tag!("//"), take_until!("\n"))) |
    complete!(delimited!(tag!("/*"), take_until!("*/"), tag!("*/"))) |
    eat_separator!(&b" \t"[..])
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

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
enum Define {
    Replace(Vec<Token>, String),
    Func(DefineFunc),
}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
struct IfCondSession {
    cond: Option<Box<Expression>>,
    session: Vec<MacroSession>,
}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
enum MacroSession {
    Define(Identifier, Define),
    Undefine(Identifier),
    IfDefine(Identifier, bool, Vec<MacroSession>, Vec<MacroSession>),
    IfCond(Vec<IfCondSession>),
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

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
struct DefineFunc {
    positions: Vec<Option<usize>>,
    nargs: usize,
    tokens: Vec<Token>,
}

impl DefineFunc {
    fn new(args: Vec<Token>, tokens: Vec<Token>) -> DefineFunc {
        let mut positions = vec![None; tokens.len()];

        for (tidx, t) in tokens.iter().enumerate() {
            for (i, a) in args.iter().enumerate() {
                if a == t {
                    positions[tidx] = Some(i);
                }
            }
        }

        assert_eq!(positions.len(), tokens.len());

        DefineFunc {
            positions: positions,
            nargs: args.len(),
            tokens: tokens,
        }
    }

    fn apply<T>(
        &self,
        name: &String,
        mut iter: T,
        result: &mut Vec<Token>,
    ) -> Result<T, PreprocessError>
    where
        T: IntoIterator<Item = Token>,
        T: Iterator<Item = Token>,
    {
        let mut stack = 0;
        let mut curret_params = Vec::new();
        let mut params: Vec<Vec<Token>> = Vec::new();

        while let Some(t) = iter.next() {
            if let Token::Operator(oper, _) = t {
                match oper {
                    Operator::LeftParen => {
                        if stack > 0 {
                            curret_params.push(t)
                        }
                        stack += 1;
                    }
                    Operator::RightParen => {
                        stack -= 1;
                        if stack == 0 {
                            params.push(curret_params);
                            break;
                        }
                        curret_params.push(t);
                    }
                    Operator::Comma if stack == 1 => {
                        params.push(curret_params);
                        curret_params = Vec::new();
                    }

                    _ => {
                        curret_params.push(t);
                    }
                }
            } else {
                curret_params.push(t);
            }
        }

        if params.len() != self.nargs {
            return Err(PreprocessError(format!(
                "Fail to apply define macro for {}, expects {} args, given {} args",
                name,
                self.nargs,
                params.len()
            )));
        }

        for (i, target) in self.tokens.iter().enumerate() {
            match self.positions[i] {
                Some(argidx) => {
                    result.extend(params[argidx].clone());
                }
                None => {
                    result.push(target.clone());
                }
            }
        }

        return Ok(iter);
    }
}

fn define_parser<'a>(input: CompleteStr<'a>) -> IResult<CompleteStr<'a>, MacroSession> {
    if let Ok((i, e)) = expression(input.clone()) {
        // We only handle function call type
        if let Expression::FunctionCall(tt, eargs) = e {
            let mut def_args = Vec::new();

            let nargs = eargs.len();
            // and we only handle identifiers
            for earg in eargs.into_iter() {
                if let Expression::Identifier(arg) = earg {
                    def_args.push(Token::Identifier(arg.clone(), arg));
                }
            }

            // if all are identifiers
            if nargs == def_args.len() {
                let tokens_r = many0!(i, token);

                if let Ok((remain, tokens)) = tokens_r {
                    let tts = tt.to_string();
                    let r = Define::Func(DefineFunc::new(def_args, tokens));

                    return Ok((remain, MacroSession::Define(tts, r)));
                }
            }
        }
    }

    do_parse!(
        input,
        key: spe!(identifier) >> tokens_str: recognize!(many0!(token)) >> ({
            let s: String = tokens_str.0.clone().into();
            let tokens = many0!(tokens_str, token).unwrap().1;

            MacroSession::Define(key, Define::Replace(tokens, s))
        })
    )
}

named!(define_macro<CS, MacroSession>, 
    do_parse!(
        spe!(char!('#')) >>         
        tag_no_case!("define") >> 
        ms: spe!(call!(define_parser)) >>
        (ms)
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

fn define_condition<'a>(input: CompleteStr<'a>) -> IResult<CS, Expression> {
    let r = recognize!(input, many0!(token))?;
    let e = expression(r.1);

    match e {
        Ok((_i, v)) => Ok((r.0, v)),
        Err(e) => Err(e),
    }
}

named!( 
    define_defined<CS, Expression>,
        do_parse!(
            spe!(tag_no_case!("defined")) >>
            spe!(opt!(tag!("("))) >>
            name: identifier >>
            spe!(opt!(tag!(")"))) >>
            (
                Expression::FunctionCall(
                    BasicType::TypeName("defined".to_string()), 
                    vec![Expression::Identifier(name)]
                )
            )
        )
);

named!( 
    define_expression<CS, Expression>,
    alt!(
        define_condition |
        define_defined 
    )
);

#[cfg_attr(rustfmt, rustfmt_skip)] 
named!(
    ifcond_macro<CS, MacroSession>, 
    do_parse!(            
        // if part
        first: do_parse!(
            spe!(char!('#')) >>
            tag_no_case!("if") >>
            e: spe!(define_expression) >>        
            line_ending >>                        
            session: many0!(terminated!(parse_macro, line_ending)) >>
            (
                IfCondSession{
                    cond: Some(Box::new(e)),
                    session: session
                }
            )
        ) >>
        // elif                
        elifsessions: many0!(do_parse!(
            spe!(char!('#')) >>
            tag_no_case!("elif") >>
            e: spe!(define_expression) >>        
            line_ending >>                        
            session: many0!(terminated!(parse_macro, line_ending)) >>
            (
                IfCondSession{
                    cond: Some(Box::new(e)),
                    session: session
                }
            )
        )) >>        
        // else part (optional)
        last: opt!(
            do_parse!(
                spe!(char!('#')) >>
                spe!(tag_no_case!("else")) >>        
                line_ending >>                  
                session: many0!(terminated!(parse_macro, line_ending)) >>
                (
                    IfCondSession {
                        cond: None,
                        session: session
                    }
                )                
            )
        ) >>
        
        spe!(return_error!(ErrorKind::Custom(1), char!('#'))) >>
        spe!(return_error!(ErrorKind::Custom(1), tag_no_case!("endif"))) >>            
        
        ({
            let mut sessions = Vec::new();
            sessions.push(first);
            sessions.extend(elifsessions);
            if let Some(l) = last {
                sessions.push(l);
            }

            MacroSession::IfCond(sessions)
        })
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
        ifcond_macro |
        ignored_macro |
        define_macro |
        undef_macro |
        empty_macro |
        normal_macro |        
        value!(MacroSession::EmptyLine,space)
    )
);

#[rustfmt_skip] 
named!(
    #[allow(unused_imports)],  // fix value! warning
    preprocess_parser <CS,Vec<MacroSession>>,
    exact!(do_parse!(
        tts: many0!(alt!(
            value!(MacroSession::EmptyLine, line_ending) |
            terminated!(parse_macro, line_ending)
        )) >> 
        last: opt!(parse_macro) >> 
        (append_vec(tts,last))
    ))
);

#[derive(Default, Debug, Clone)]
struct PreprocessState {
    defines: HashMap<String, Define>,
    normal_tokens: Vec<String>,
}

impl PreprocessState {
    pub fn get_defined(&self, s: &String) -> Option<Define> {
        match self.defines.get(s) {
            Some(def) => Some(def.clone()),
            None => None,
        }
    }
}

impl EvalContext for PreprocessState {
    fn get(&self, i: &Identifier) -> Option<Constant> {
        let r = self.defines.get(i);
        if r.is_none() {
            return None;
        }

        match r.unwrap() {
            &Define::Replace(_, ref s) => {
                let e = expression(CompleteStr(s));
                if let Ok((_, v)) = e {
                    return v.eval_constant(self).ok();
                }
            }
            _ => (),
        }

        None
    }

    fn defined(&self, i: &Identifier) -> bool {
        self.defines.contains_key(i)
    }
}

fn preprocess_source_line(
    tt: Vec<Token>,
    state: &PreprocessState,
) -> Result<(bool, Vec<Token>), PreprocessError> {
    let mut result = Vec::new();
    let mut processed = false;
    let mut iter = tt.into_iter();

    while let Some(t) = iter.next() {
        use self::Define::*;

        match t {
            Token::Identifier(ref s, ..) => {
                if let Some(def) = state.get_defined(s) {
                    processed = true;
                    match def {
                        Replace(mut childs, _) => {
                            result.append(&mut childs);
                        }
                        Func(def_func) => {
                            iter = def_func.apply(s, iter, &mut result)?;
                        }
                    }
                } else {
                    result.push(t.clone())
                }
            }
            _ => result.push(t),
        }
    }

    Ok((processed, result))
}

fn preprocess_token(tt: Token, state: &mut PreprocessState) {
    match tt {
        Token::Identifier(s, ..) => {
            state.normal_tokens.push(s);
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

fn is_condition_true(c: Constant) -> bool {
    match c {
        Constant::Integer(i) => i != 0,
        Constant::Bool(b) => b,
        Constant::Float(f) => f != 0.0,
    }
}

impl From<EvalError> for PreprocessError {
    fn from(e: EvalError) -> PreprocessError {
        PreprocessError(e.into_string())
    }
}

fn preprocess_session(s: MacroSession, state: &mut PreprocessState) -> Result<(), PreprocessError> {
    match s {
        MacroSession::EmptyLine => (),
        MacroSession::Empty => (),
        MacroSession::Define(s, a) => {
            state.defines.insert(s, a);
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
                    preprocess_session(child, state)?;
                }
            } else {
                for child in second.into_iter() {
                    preprocess_session(child, state)?;
                }
            }
        }
        MacroSession::IfCond(sessions) => for s in sessions {
            let do_session = if let Some(cond) = s.cond {
                let c = cond.eval_constant(state)?;
                is_condition_true(c)
            } else {
                true
            };

            if do_session {
                for child in s.session.into_iter() {
                    preprocess_session(child, state)?;
                }
                break;
            }
        },
        MacroSession::Normal(n) => {
            let (mut processed, mut tokens) = preprocess_source_line(n, state)?;
            while processed {
                let (p, tts) = preprocess_source_line(tokens, state)?;
                processed = p;
                tokens = tts;
            }

            for tokens in tokens.into_iter() {
                preprocess_token(tokens, state)
            }

            state.normal_tokens.push("\n".into());
        }
    }

    Ok(())
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
/// #if
/// #elif
/// defined
///
/// Ignored :
///
/// #line
/// #version
/// #extension
/// #pragma
///
///

pub fn preprocess(s: &str, predefs: &HashMap<String, String>) -> Result<String, PreprocessError> {
    let stage0 = lines(CompleteStr(s))?.1;
    let stage1 = remove_comment(CompleteStr(&stage0))?.1;
    let sessions = preprocess_parser(CompleteStr(&stage1));

    let sessions = sessions?.1;
    let mut state = PreprocessState::default();

    // append predefs
    for (k, v) in predefs.iter() {
        let whole_line = format!("#define {} {}", k, v);
        let m = parse_macro(CompleteStr(&whole_line))?;

        match m.1 {
            MacroSession::Define(s, a) => {
                state.defines.insert(s, a);
            }
            _ => (),
        };
    }

    for session in sessions.into_iter() {
        preprocess_session(session, &mut state)?;
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
    fn preprocess_test_stage0and1() {
        let test_text = r#"void main(void) {
    vFragPos = vec3(uMMatrix * vec4(aVertexPosition, 1.0));            
    vNormal = mat3(uNMatrix) * aVertexNormal;
    vTexCoords = aTextureCoord;

    gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 1.0);
}"#;
        let stage0 = lines(CompleteStr(test_text)).unwrap().1;
        assert_eq!(
            format!("{:?}", stage0),
            "\"void main(void) {\\n    vFragPos = vec3(uMMatrix * vec4(aVertexPosition, 1.0));\\n    vNormal = mat3(uNMatrix) * aVertexNormal;\\n    vTexCoords = aTextureCoord;\\n\\n    gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 1.0);\\n}\""
        );

        let i = remove_comment(CompleteStr(&stage0));

        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "\" void main(void) { \\n vFragPos = vec3(uMMatrix * vec4(aVertexPosition, 1.0)); \\n vNormal = mat3(uNMatrix) * aVertexNormal; \\n vTexCoords = aTextureCoord; \\n \\n gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 1.0); \\n }\""
        );
    }

    #[test]
    fn preprocess_test_file() {
        let test_text = include_str!("../data/test/preprocessor_test.glsl");
        let expect_result = include_str!("../data/test/preprocessor_test_result.glsl");
        let r = preprocess(test_text, &HashMap::new()).unwrap();

        //Write result to temp directory.
        // use std::fs::File;
        // use std::io::prelude::*;
        // let mut file = File::create("D:\\Temp\\preprocessor_test_result.glsl").unwrap();
        // file.write_all(&r.as_bytes()).unwrap();

        assert_eq!(r, expect_result);
    }

    #[test]
    fn preprocess_func_macro() {
        let test_text = r#" #define F(A,B)   A+B+B+A
            F((1+3),2)
        "#;

        let r = preprocess(test_text, &HashMap::new()).unwrap();

        assert_eq!(r.trim(), "( 1 + 3 ) + 2 + 2 + ( 1 + 3 )");
    }

    #[test]
    fn preprocess_def_cond() {
        let test_text = r#" #if 0
            A
        #else
            B
        #endif
        "#;

        let r = preprocess(test_text, &HashMap::new()).unwrap();

        assert_eq!(r.trim(), "B");
    }

    #[test]
    fn preprocess_def_cond_ifelif() {
        let test_text = r#" #if C1 > 3
            A
        #elif C2 <= 5
            B
        #else
            C
        #endif
        "#;

        let mut hm: HashMap<String, String> = HashMap::new();
        hm.insert("C1".into(), "1".to_string());
        hm.insert("C2".into(), "4".to_string());
        let r = preprocess(test_text, &hm).unwrap();

        assert_eq!(r.trim(), "B");
    }

    #[test]
    fn preprocess_def_defined() {
        let test_text = r#" #if defined(K)
            A
        #elif C2 <= 5
            B
        #else
            C
        #endif
        "#;

        let mut hm: HashMap<String, String> = HashMap::new();
        hm.insert("K".into(), "".to_string());
        let r = preprocess(test_text, &hm).unwrap();
        assert_eq!(r.trim(), "A");
    }

    #[test]
    fn parse_ifcond() {
        let test_text = r#" #if 0
            A
        #else 
            B
        #endif
        "#;

        let i = ifcond_macro(CompleteStr(test_text));

        assert_eq!(
            format!("{:?}", i.unwrap().1),
            "IfCond([IfCondSession { cond: Some(Constant(Integer(0))), session: [Normal([Identifier(\"A\", \"A\")])] }, IfCondSession { cond: None, session: [Normal([Identifier(\"B\", \"B\")])] }])"
        );
    }

}
