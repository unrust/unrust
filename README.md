# unigame
A simple webgl physics (wasm32-unknown-unknown) demo 

[Live demo](https://edwin0cheng.github.io/unigame_demo/)

This project is my first try on rust to test the possiblity of wasm32-unknown-unknown target.

Crates Used :

* nphysics3d + ncollide (with minor cargo replace trick to let it build on wasm32, see Cargo.toml)
* Some code snippet from https://github.com/oussama/webgl-rs project and https://github.com/oussama/glenum-rs.git
* stdweb


## Build 
### As web app (wasm32-unknown-unknown)

The target `wasm32-unknown-unknown` is currently only on the nightly builds as of Jan-30 2018. 

```
cargo install cargo-web # installs web sub command
rustup override set nightly
rustup target install wasm32-unknown-unknown
cargo web start --target wasm32-unknown-unknown --release
```

### As desktop app (native-opengl)
```
rustup override set nightly
cargo run --release
```
