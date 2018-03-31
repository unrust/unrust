use engine::asset::loader::{Loadable, Loader};
use engine::asset::{AssetError, AssetResult, AssetSystem, File, FileFuture};
use engine::render::{PreprocessedShaderCode, Shader, ShaderKindFs, ShaderKindProvider,
                     ShaderKindVs};

use std::str;
use std::marker::PhantomData;
use futures::Future;

pub struct ShaderLoader<T: ShaderKindProvider> {
    phantom: PhantomData<T>,
}

impl<T> Loader<Shader<T>> for ShaderLoader<T>
where
    T: ShaderKindProvider,
{
    fn load<A>(_asys: A, mut file: Box<File>) -> AssetResult<Shader<T>> {
        let buf = file.read_binary()
            .map_err(|_| AssetError::ReadBufferFail(file.name()))?;
        let s = str::from_utf8(&buf).map_err(|e| AssetError::InvalidFormat {
            path: file.name(),
            len: buf.len(),
            reason: format!("{:?}", e),
        })?;

        let code = PreprocessedShaderCode::new(T::kind(), &file.name(), s).unwrap();
        Ok(Shader::<T>::from_preprocessed(&file.name(), code))
    }
}

impl<T: ShaderKindProvider> Loadable for Shader<T> {
    type Loader = ShaderLoader<T>;

    fn load_future<A>(asys: A, f0: FileFuture) -> Box<Future<Item = Self, Error = AssetError>>
    where
        Self: 'static,
        A: AssetSystem + Clone + 'static,
    {
        // futurize
        Box::new(f0.then(move |r| {
            let f = r.map_err(|e| AssetError::FileIoError(e))?;
            Self::load(asys, f)
        }))
    }
}

pub type ShaderVSLoader = ShaderLoader<ShaderKindVs>;
pub type ShaderFSLoader = ShaderLoader<ShaderKindFs>;
