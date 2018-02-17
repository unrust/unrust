use nom::types::CompleteStr;
use nom::sp;

// Parser rewriter, discarding optional whitespaces
named!(#[allow(dead_code)], pub ospace<CompleteStr, Option<CompleteStr>>, opt!(sp));

#[allow(unused_macros)]
macro_rules! ows {
  ($i:expr, $($args:tt)*) => {{
    sep!($i, $crate::macros::ospace, $($args)*)
  }}
}
