use na::{Matrix4, Vector3};
use std::fmt::Debug;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::mem::size_of;

use webgl::{WebGLRenderingContext, WebGLUniformLocation};

trait IntoBytes {
    fn into_bytes(&self) -> Vec<u8>;
}

impl<T: Clone> IntoBytes for [T] {
    fn into_bytes(&self) -> Vec<u8> {
        let v = self.to_vec();
        let len = size_of::<T>() * v.len();
        unsafe {
            let slice = v.into_boxed_slice();
            Vec::<u8>::from_raw_parts(Box::into_raw(slice) as _, len, len)
        }
    }
}

pub trait UniformAdapter: Debug {
    fn set(&self, gl: &WebGLRenderingContext, loc: &WebGLUniformLocation);

    fn to_hash(&self) -> u64;
}

impl UniformAdapter for Matrix4<f32> {
    fn set(&self, gl: &WebGLRenderingContext, loc: &WebGLUniformLocation) {
        let m = *self;
        gl.uniform_matrix_4fv(&loc, &m.into());
    }

    fn to_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        s.write(&self.as_slice().into_bytes());
        s.finish()
    }
}

impl UniformAdapter for f32 {
    fn set(&self, gl: &WebGLRenderingContext, loc: &WebGLUniformLocation) {
        gl.uniform_1f(&loc, *self);
    }

    fn to_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        let f: f32 = *self;
        s.write(&[f].into_bytes());
        s.finish()
    }
}

impl UniformAdapter for i32 {
    fn set(&self, gl: &WebGLRenderingContext, loc: &WebGLUniformLocation) {
        gl.uniform_1i(&loc, *self);
    }

    fn to_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        s.write_i32(*self);
        s.finish()
    }
}

impl UniformAdapter for Vector3<f32> {
    fn set(&self, gl: &WebGLRenderingContext, loc: &WebGLUniformLocation) {
        gl.uniform_3f(&loc, (self.x, self.y, self.z));
    }

    fn to_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        s.write(&self.as_slice().into_bytes());
        s.finish()
    }
}
