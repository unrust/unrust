# unigame
A simple webgl physic (wasm32-unknown-unknown) demo 

This project is my first try on rust to test the possiblity of wasm32-unknown-unknown target.

Used crates :

* nphysics3d + ncollide (with minor cargo replace trick to let it build on wasm32, see Cargo.toml)
* Some code snippet from https://github.com/oussama/webgl-rs project and https://github.com/oussama/glenum-rs.git
* stdweb


## Build 
```
cargo web start --target-webasm --release
```
