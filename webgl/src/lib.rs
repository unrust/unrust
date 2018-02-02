#![allow(dead_code)]
#![feature(nll)]
#![recursion_limit = "512"]

extern crate glenum;

#[macro_use]
extern crate stdweb;

pub mod webgl;
pub use glenum::*;

pub mod common {
    use std::ops::Deref;
    type Reference = i32;

    #[cfg(not(target_arch = "wasm32"))]
    use webgl_native::*;

    #[derive(Debug, PartialEq)]
    pub struct GLContext {
        pub reference: Reference,
    }

    #[derive(Debug)]
    pub struct WebGLRenderingContext {
        pub common: GLContext,
    }

    impl From<GLContext> for Reference {
        fn from(w: GLContext) -> Reference {
            w.reference
        }
    }

    impl Deref for WebGLRenderingContext {
        type Target = GLContext;
        fn deref(&self) -> &GLContext {
            &self.common
        }
    }

    #[derive(Debug)]
    pub struct WebGLBuffer(pub Reference);

    impl Deref for WebGLBuffer {
        type Target = Reference;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug)]
    pub struct WebGLShader(pub Reference);
    impl Deref for WebGLShader {
        type Target = Reference;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct WebGLProgram(pub i32);
    impl Deref for WebGLProgram {
        type Target = i32;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct WebGLTexture(pub Reference);
    impl Deref for WebGLTexture {
        type Target = Reference;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug)]
    pub struct WebGLVertexArray(pub Reference);
    impl Deref for WebGLVertexArray {
        type Target = Reference;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct WebGLUniformLocation(pub i32);
    impl Deref for WebGLUniformLocation {
        type Target = i32;
        fn deref(&self) -> &i32 {
            &self.0
        }
    }

}

pub use self::common::*;
