use engine::asset::{AssetError, File, FileFuture};
use futures::prelude::*;

pub trait Loader<U>
where
    U: Loadable,
{
    fn load(f: Box<File>) -> Result<U, AssetError>;
}

pub trait Loadable: Sized
where
    Self::Loader: Loader<Self>,
{
    type Loader;

    fn load(f: Box<File>) -> Result<Self, AssetError> {
        Self::Loader::load(f)
    }

    fn load_future(f0: FileFuture) -> Box<Future<Item = Self, Error = AssetError>>
    where
        Self: 'static,
    {
        // futurize
        Box::new(f0.then(move |r| {
            let f = r.map_err(|e| AssetError::FileIoError(e))?;
            Self::load(f)
        }))
    }
}
