use engine::asset::{AssetError, AssetSystem, File, FileFuture};
use futures::prelude::*;

pub trait Loader<U>
where
    U: Loadable,
{
    fn load<A>(asys: A, f: Box<File>) -> Result<U, AssetError>
    where
        A: AssetSystem + Clone;
}

pub trait Loadable: Sized
where
    Self::Loader: Loader<Self>,
{
    type Loader;

    fn load<A: AssetSystem + Clone>(asys: A, f: Box<File>) -> Result<Self, AssetError> {
        Self::Loader::load(asys, f)
    }

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
