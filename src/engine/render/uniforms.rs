use math::{Matrix4, Vector2, Vector3, Vector4};
use std::mem::size_of;

use std::rc::Rc;
use std::cell::RefCell;
use std::rc;
use fnv::FnvHashMap;

use webgl::{WebGLProgram, WebGLRenderingContext, WebGLUniformLocation};
use engine::render::Texture;

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

#[derive(Debug, Clone)]
pub struct UniformTexture(rc::Weak<Texture>, u32);

impl PartialEq for UniformTexture {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1 && {
            match (self.0.upgrade(), other.0.upgrade()) {
                (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
                _ => false,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UniformAdapter {
    Matrix4(Matrix4<f32>),
    F32(f32),
    Bool(bool),
    I32(i32),
    Vector2(Vector2<f32>),
    Vector3(Vector3<f32>),
    Vector4(Vector4<f32>),
    Texture(UniformTexture),
}

macro_rules! impl_from_uniform_adapter {
    ($t: ty, $id: ident) => {
        impl From<$t> for UniformAdapter {
            fn from(m: $t) -> UniformAdapter {
                UniformAdapter::$id(m)
            }
        }
    };
}

impl_from_uniform_adapter!(Matrix4<f32>, Matrix4);
impl_from_uniform_adapter!(f32, F32);
impl_from_uniform_adapter!(bool, Bool);
impl_from_uniform_adapter!(i32, I32);
impl_from_uniform_adapter!(Vector2<f32>, Vector2);
impl_from_uniform_adapter!(Vector3<f32>, Vector3);
impl_from_uniform_adapter!(Vector4<f32>, Vector4);

impl From<(rc::Weak<Texture>, u32)> for UniformAdapter {
    fn from(m: (rc::Weak<Texture>, u32)) -> UniformAdapter {
        UniformAdapter::Texture(UniformTexture(m.0, m.1))
    }
}

impl UniformAdapter {
    fn set(&self, gl: &WebGLRenderingContext, loc: &WebGLUniformLocation) {
        match *self {
            UniformAdapter::Matrix4(m) => {
                gl.uniform_matrix_4fv(&loc, &m.into());
            }
            UniformAdapter::F32(f) => {
                gl.uniform_1f(&loc, f);
            }
            UniformAdapter::Bool(b) => {
                gl.uniform_1i(&loc, if b { 1 } else { 0 });
            }
            UniformAdapter::I32(i) => {
                gl.uniform_1i(&loc, i);
            }
            UniformAdapter::Vector2(v) => {
                gl.uniform_2f(&loc, (v.x, v.y));
            }
            UniformAdapter::Vector3(v) => {
                gl.uniform_3f(&loc, (v.x, v.y, v.z));
            }
            UniformAdapter::Vector4(v) => {
                gl.uniform_4f(&loc, (v.x, v.y, v.z, v.w));
            }
            UniformAdapter::Texture(ref v) => {
                gl.uniform_1i(&loc, v.1 as i32);
            }
        }
    }
}

// #[derive(Debug)]
// struct UniformAdapterEntry {
//     adapter: UniformAdapter,
//     commited: bool,
// }

#[derive(Default, Debug)]
pub struct UniformCache {
    uniform_entries: RefCell<FnvHashMap<String, UniformAdapter>>,
    pending_entries: RefCell<FnvHashMap<String, UniformAdapter>>,

    uniform_map: RefCell<FnvHashMap<String, Option<Rc<WebGLUniformLocation>>>>,
}

impl UniformCache {
    pub fn set<T>(&self, s: &str, data: T)
    where
        T: Into<UniformAdapter>,
    {
        use std::collections::hash_map::Entry;

        let mut entries = self.uniform_entries.borrow_mut();
        let adapter = data.into();

        let entry = entries.entry(s.to_owned());

        if let Entry::Occupied(o) = entry {
            if *o.get() == adapter {
                return;
            }
            let (key, _) = o.remove_entry();
            self.pending_entries.borrow_mut().insert(key, adapter);
        } else {
            self.pending_entries
                .borrow_mut()
                .insert(s.to_owned(), adapter);
        }
    }

    pub fn commit(&self, gl: &WebGLRenderingContext, prog: &WebGLProgram) {
        {
            let mut pending = self.pending_entries.borrow_mut();
            let entries = pending.drain();
            let mut uniforms = self.uniform_entries.borrow_mut();

            for (key, adapter) in entries.into_iter() {
                if let Some(u) = self.get_uniform(gl, prog, &key) {
                    adapter.set(gl, &u);
                }

                uniforms.insert(key, adapter);
            }
        }
    }

    fn get_uniform(
        &self,
        gl: &WebGLRenderingContext,
        prog: &WebGLProgram,
        s: &str,
    ) -> Option<Rc<WebGLUniformLocation>> {
        let mut m = self.uniform_map.borrow_mut();

        match m.get(s) {
            Some(ref u) => u.as_ref().map(|x| x.clone()),
            None => {
                let uloc = gl.get_uniform_location(&prog, s.into());

                match uloc {
                    None => {
                        m.insert(s.into(), None);
                        None
                    }
                    Some(uloc) => {
                        let p = Rc::new(uloc);
                        m.insert(s.into(), Some(p.clone()));
                        Some(p)
                    }
                }
            }
        }
    }
}
