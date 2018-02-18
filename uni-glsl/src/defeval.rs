use expression::{BinaryOp, Expression};
use token::{Constant, Identifier};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug)]
pub struct EvalError(String);

impl EvalError {
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<&'static str> for EvalError {
    fn from(s: &'static str) -> EvalError {
        EvalError(s.into())
    }
}

pub trait EvalContext: Debug + Clone {
    fn get(&self, i: &Identifier) -> Option<Constant>;

    fn defined(&self, i: &Identifier) -> bool;

    fn value(&self, i: &Identifier) -> Constant {
        match self.get(i) {
            Some(c) => bool_constant_to_integer(c),
            None => Constant::Integer(0),
        }
    }
}

#[derive(Debug, Default, Clone)]
struct EvalContextSimple {
    variables: HashMap<Identifier, Constant>,
}

impl EvalContext for EvalContextSimple {
    fn get(&self, i: &Identifier) -> Option<Constant> {
        self.variables.get(i).map(|c| c.clone())
    }

    fn defined(&self, i: &Identifier) -> bool {
        self.variables.contains_key(i)
    }
}

fn bool_constant_to_integer(c: Constant) -> Constant {
    match c {
        Constant::Bool(b) => Constant::Integer(if b { 1 } else { 0 }),
        a => a,
    }
}

fn is_bool(c: &Constant) -> bool {
    match c {
        &Constant::Bool(_) => true,
        _ => false,
    }
}

fn is_float(c: &Constant) -> bool {
    match c {
        &Constant::Float(_) => true,
        _ => false,
    }
}

fn to_float(c: &Constant) -> f32 {
    match c {
        &Constant::Float(f) => f,
        &Constant::Integer(i) => i as f32,
        &Constant::Bool(_) => 0.0,
    }
}

fn to_int(c: &Constant) -> i64 {
    match c {
        &Constant::Float(f) => f as i64,
        &Constant::Integer(i) => i,
        &Constant::Bool(_) => 0,
    }
}

const BOOL_ARITH_ERROR: &str = "bool arithmetic syntax is not supported in constant expression";

pub trait Eval {
    fn eval_constant<T>(&self, ctx: &T) -> Result<Constant, EvalError>
    where
        T: EvalContext;
}

macro_rules! eval_const_op {
    ($ctx:ident, $e1:ident, $e2:ident, $op:tt) => {{
        let v1 = $e1.eval_constant($ctx)?;
        let v2 = $e2.eval_constant($ctx)?;

        if is_bool(&v1) || is_bool(&v2) {
            return Err(BOOL_ARITH_ERROR.into());
        }

        if (is_float(&v1) && !is_float(&v2)) || (!is_float(&v1) && is_float(&v2)) {
            return Ok( Constant::Float( to_float(&v1) $op to_float(&v2)));
        }

        match v1 {
            Constant::Integer(i) => Ok(Constant::Integer(i $op to_int(&v2))),
            Constant::Float(f) => Ok(Constant::Float(f $op to_float(&v2))),
            _ => unreachable!(),
        }
    }};
}

macro_rules! eval_const_op_int {
    ($ctx:ident, $e1:ident, $e2:ident, $op:tt) => {{
        let v1 = $e1.eval_constant($ctx)?;
        let v2 = $e2.eval_constant($ctx)?;

        if is_bool(&v1) || is_bool(&v2) {
            return Err(BOOL_ARITH_ERROR.into());
        }

        match v1 {
            Constant::Integer(i) => Ok(Constant::Integer(i $op to_int(&v2))),
            Constant::Float(f) => Ok(Constant::Integer((f as i64) $op to_int(&v2))),
            _ => unreachable!(),
        }
    }};
}

macro_rules! eval_const_op_bool_int {
    ($ctx:ident, $e1:ident, $e2:ident, $op:tt) => {{
        let v1 = $e1.eval_constant($ctx)?;
        let v2 = $e2.eval_constant($ctx)?;

        if is_bool(&v1) || is_bool(&v2) {
            return Err(BOOL_ARITH_ERROR.into());
        }

        if (is_float(&v1) && !is_float(&v2)) || (!is_float(&v1) && is_float(&v2)) {
            return Ok( Constant::Integer( (to_float(&v1) $op to_float(&v2)) as i64 ));
        }

        match v1 {
            Constant::Integer(i) => Ok(Constant::Integer( (i $op to_int(&v2)) as i64 )),
            Constant::Float(f) => Ok(Constant::Integer( (f $op to_float(&v2)) as i64 )),
            _ => unreachable!(),
        }
    }};
}

macro_rules! eval_const_op_cond {
    ($ctx:ident, $e1:ident, $e2:ident, $op:tt) => {{
        let v1 = $e1.eval_constant($ctx)?;
        let v2 = $e2.eval_constant($ctx)?;

        if is_bool(&v1) || is_bool(&v2) {
            return Err(BOOL_ARITH_ERROR.into());
        }

        match v1 {
            Constant::Integer(i) => Ok(Constant::Integer( ((i != 0) $op (to_int(&v2) != 0))  as i64)),
            Constant::Float(f) => Ok(Constant::Integer(  (((f as i64) != 0) $op (to_int(&v2) != 0)) as i64 )),
            _ => unreachable!(),
        }
    }};
}

impl Eval for Expression {
    fn eval_constant<T>(&self, ctx: &T) -> Result<Constant, EvalError>
    where
        T: EvalContext,
    {
        match self {
            &Expression::Empty => Ok(Constant::Integer(0)),
            &Expression::Identifier(ref i) => Ok(ctx.value(i)),
            &Expression::Constant(ref c) => Ok(bool_constant_to_integer(c.clone())),
            &Expression::Bracket(_, ..) => {
                Err("bracket syntax is not supported in constant expression".into())
            }
            &Expression::FunctionCall(ref tn, ref args) => {
                if tn.to_string().to_lowercase() == "defined" {
                    if args.len() == 1 {
                        if let Expression::Identifier(ref ident) = args[0] {
                            return if ctx.defined(ident) {
                                Ok(Constant::Integer(1))
                            } else {
                                Ok(Constant::Integer(0))
                            };
                        }
                    }
                }

                Err("function call syntax is not supported in constant expression".into())
            }
            &Expression::DotField(_, ..) => {
                Err("dot syntax is not supported in constant expression".into())
            }
            &Expression::PostInc(ref e) => {
                let v = e.eval_constant(ctx)?;

                match v {
                    Constant::Bool(_) => Err(BOOL_ARITH_ERROR.into()),
                    Constant::Integer(i) => Ok(Constant::Integer(i + 1)),
                    Constant::Float(f) => Ok(Constant::Float(f + 1.0)),
                }
            }
            &Expression::PostDec(ref e) => {
                let v = e.eval_constant(ctx)?;
                match v {
                    Constant::Bool(_) => Err(BOOL_ARITH_ERROR.into()),
                    Constant::Integer(i) => Ok(Constant::Integer(i - 1)),
                    Constant::Float(f) => Ok(Constant::Float(f - 1.0)),
                }
            }
            &Expression::PreInc(ref e) => {
                let v = e.eval_constant(ctx)?;

                match v {
                    Constant::Bool(_) => Err(BOOL_ARITH_ERROR.into()),
                    Constant::Integer(i) => Ok(Constant::Integer(i + 1)),
                    Constant::Float(f) => Ok(Constant::Float(f + 1.0)),
                }
            }
            &Expression::PreDec(ref e) => {
                let v = e.eval_constant(ctx)?;
                match v {
                    Constant::Bool(_) => Err(BOOL_ARITH_ERROR.into()),
                    Constant::Integer(i) => Ok(Constant::Integer(i - 1)),
                    Constant::Float(f) => Ok(Constant::Float(f - 1.0)),
                }
            }
            &Expression::Plus(ref e) => {
                let v = e.eval_constant(ctx)?;
                match v {
                    Constant::Bool(_) => Err(BOOL_ARITH_ERROR.into()),
                    Constant::Integer(i) => Ok(Constant::Integer(i)),
                    Constant::Float(f) => Ok(Constant::Float(f)),
                }
            }
            &Expression::Minus(ref e) => {
                let v = e.eval_constant(ctx)?;
                match v {
                    Constant::Bool(_) => Err(BOOL_ARITH_ERROR.into()),
                    Constant::Integer(i) => Ok(Constant::Integer(-i)),
                    Constant::Float(f) => Ok(Constant::Float(-f)),
                }
            }

            &Expression::Not(ref e) => {
                let v = e.eval_constant(ctx)?;
                match v {
                    Constant::Bool(b) => Ok(Constant::Bool(!b)),
                    Constant::Integer(i) => Ok(Constant::Integer(if i == 0 { 1 } else { 0 })),
                    Constant::Float(f) => Ok(Constant::Integer(if f == 0.0 { 1 } else { 0 })),
                }
            }

            &Expression::Tilde(ref e) => {
                let v = e.eval_constant(ctx)?;
                match v {
                    Constant::Bool(b) => Ok(Constant::Bool(!b)),
                    Constant::Integer(i) => Ok(Constant::Integer(!i)),
                    Constant::Float(f) => Ok(Constant::Integer(if f == 0.0 { 1 } else { 0 })),
                }
            }

            &Expression::Binary(ref binop, ref e1, ref e2) => match binop {
                &BinaryOp::Or => eval_const_op_cond!(ctx, e1, e2, ||),
                &BinaryOp::And => eval_const_op_cond!(ctx, e1, e2, &&),
                &BinaryOp::Equal => eval_const_op_bool_int!(ctx, e1, e2, ==),
                &BinaryOp::NonEqual => eval_const_op_bool_int!(ctx, e1, e2, !=),
                &BinaryOp::LT => eval_const_op_bool_int!(ctx, e1, e2, <),
                &BinaryOp::GT => eval_const_op_bool_int!(ctx, e1, e2, >),
                &BinaryOp::LTE => eval_const_op_bool_int!(ctx, e1, e2, <=),
                &BinaryOp::GTE => eval_const_op_bool_int!(ctx, e1, e2, >=),

                &BinaryOp::RShift => eval_const_op_int!(ctx, e1, e2, >>),
                &BinaryOp::LShift => eval_const_op_int!(ctx, e1, e2, <<),

                &BinaryOp::Add => eval_const_op!(ctx, e1, e2, +),
                &BinaryOp::Sub => eval_const_op!(ctx, e1, e2, -),
                &BinaryOp::Mult => eval_const_op!(ctx, e1, e2, *),
                &BinaryOp::Div => eval_const_op!(ctx, e1, e2, /),
                &BinaryOp::Mod => eval_const_op!(ctx, e1, e2, %),

                _ => Err("Operation not supported".into()),
            },

            &Expression::Ternary(ref e0, ref e1, ref e2) => {
                let v0 = e0.eval_constant(ctx)?;
                let v1 = e1.eval_constant(ctx)?;
                let v2 = e2.eval_constant(ctx)?;

                if to_int(&v0) != 0 {
                    Ok(v1)
                } else {
                    Ok(v2)
                }
            }

            other => Err(EvalError(format!("Not supported operator {:?}", other))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expression::expression;
    use nom::types::CompleteStr;

    #[test]
    fn parse_eval_const_simple() {
        let e = expression(CompleteStr("1+1")).unwrap().1;
        let r = e.eval_constant(&EvalContextSimple::default());

        assert_eq!(r.unwrap(), Constant::Integer(2));

        let e = expression(CompleteStr("1 == 1")).unwrap().1;
        let r = e.eval_constant(&EvalContextSimple::default());

        assert_eq!(r.unwrap(), Constant::Integer(1));

        let e = expression(CompleteStr("1 != 0")).unwrap().1;
        let r = e.eval_constant(&EvalContextSimple::default());

        assert_eq!(r.unwrap(), Constant::Integer(1));

        let e = expression(CompleteStr("1 > 0")).unwrap().1;
        let r = e.eval_constant(&EvalContextSimple::default());

        assert_eq!(r.unwrap(), Constant::Integer(1));
    }
}
