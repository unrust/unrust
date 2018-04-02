# unrust

[![Build Status](https://travis-ci.org/edwin0cheng/unrust.svg?branch=master)](https://travis-ci.org/edwin0cheng/unrust)

A pure rust based (webgl 2.0 / native) game engine

## Live Demo

* [Boxes](https://edwin0cheng.github.io/unrust/demo/boxes)
* [Sponza](https://edwin0cheng.github.io/unrust/demo/sponza)

## Build

### As web app (wasm32-unknown-unknown)

The target `wasm32-unknown-unknown` is currently only on the nightly builds as of Jan-30 2018.

```
cargo install cargo-web # installs web sub command
rustup override set nightly
rustup target install wasm32-unknown-unknown
cargo web start --example boxes --release
```

### As desktop app (native-opengl)

```
rustup override set nightly
cargo run --example boxes --release
```
