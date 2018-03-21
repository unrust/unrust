#![recursion_limit = "512"]

extern crate serde;
#[macro_use]
extern crate serde_derive;

#[cfg(not(target_arch = "wasm32"))]
extern crate gl;

#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

#[cfg(target_arch = "wasm32")]
#[path = "webgl.rs"]
pub mod webgl;

#[cfg(not(target_arch = "wasm32"))]
#[path = "webgl_native.rs"]
mod webgl;

#[cfg(not(target_arch = "wasm32"))]
pub const IS_GL_ES: bool = false;

#[cfg(target_arch = "wasm32")]
pub const IS_GL_ES: bool = true;

mod glenum;

pub use glenum::*;
pub use webgl::WebGLContext;

pub mod common {
    use std::ops::Deref;

    type Reference = super::webgl::Reference;
    type GLContext = super::webgl::GLContext;

    #[derive(Debug, Clone)]
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
    pub struct WebGLProgram(pub Reference);
    impl Deref for WebGLProgram {
        type Target = Reference;
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
    pub struct WebGLUniformLocation {
        pub reference: Reference,
        pub name: String,
    }
    impl Deref for WebGLUniformLocation {
        type Target = Reference;
        fn deref(&self) -> &Reference {
            &self.reference
        }
    }

    #[derive(Debug)]
    pub struct WebGLFrameBuffer(pub Reference);
    impl Deref for WebGLFrameBuffer {
        type Target = Reference;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    pub fn print(s: &str) {
        GLContext::print(s);
    }
}

pub use self::common::*;
