#[macro_use]
extern crate nom;

named!(pub get_greeting<&str,&str>,
    tag_s!("hi")
);

#[test]
fn parse_alpha() {
  println!("{:?}", get_greeting("hi there "));
}
