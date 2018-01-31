# unigame
A simple webgl physics (wasm32-unknown-unknown) demo 

[Live demo](https://edwin0cheng.github.io/unigame_demo/)

This project is my first try on rust to test the possiblity of wasm32-unknown-unknown target.

Crates Used :

* nphysics3d + ncollide (with minor cargo replace trick to let it build on wasm32, see Cargo.toml)
* Some code snippet from https://github.com/oussama/webgl-rs project and https://github.com/oussama/glenum-rs.git
* stdweb


## Build 
```
cargo web start --target-webasm --release
```
