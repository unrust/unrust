# unrust

[![Build Status](https://travis-ci.org/unrust/unrust.svg?branch=master)](https://travis-ci.org/unrust/unrust)

A pure rust based (webgl 2.0 / native) game engine

Current Version : 0.1.1

**This project is under heavily development, all api are very unstable until version 0.2**

## Live Demo

* [Boxes](https://edwin0cheng.github.io/unrust/demo/boxes)
* [Sponza](https://edwin0cheng.github.io/unrust/demo/sponza)
* [Sound](https://edwin0cheng.github.io/unrust/demo/sound)
* [Post-Processing](https://edwin0cheng.github.io/unrust/demo/postprocessing)
* [MeshObj](https://edwin0cheng.github.io/unrust/demo/meshobj)
* [Basic](https://edwin0cheng.github.io/unrust/demo/basic)

## Usage 

You can reference [basic.rs](https://github.com/edwin0cheng/unrust/blob/master/examples/basic.rs) for now, more documetations will be coming soon.

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

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
