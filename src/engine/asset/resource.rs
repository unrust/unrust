use std::mem;
use std::fmt::Debug;
use std::fmt;
use std::cell::{Ref, RefCell};
use futures::{Async, Future};

use engine::asset::loader;
use engine::asset::{AssetError, AssetResult};

impl<T> Debug for ResourceKind<T>
where
    T: Debug + loader::Loadable,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ResourceKind::Consumed => write!(f, "ResourceKind::Consumed"),
            &ResourceKind::Data(ref t) => write!(f, "ResourceKind::Data({:?})", *t),
            &ResourceKind::Future(_) => write!(f, "ResourceKind::Future"),
        }
    }
}

enum ResourceKind<T: Debug> {
    Consumed,
    Data(T),
    Future(Box<Future<Item = T, Error = AssetError>>),
}

impl<T: Debug> ResourceKind<T> {
    fn try_into_data(self) -> Option<T> {
        match self {
            ResourceKind::Data(d) => Some(d),
            _ => None,
        }
    }

    fn try_as_data(&self) -> Option<&T> {
        match self {
            &ResourceKind::Data(ref d) => Some(d),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Resource<T: Debug + loader::Loadable>(RefCell<ResourceKind<T>>);

impl<T: Debug + loader::Loadable> Resource<T> {
    pub fn new_future<FT>(f: FT) -> Self
    where
        FT: Future<Item = T, Error = AssetError> + 'static,
    {
        Resource(RefCell::new(ResourceKind::Future(Box::new(f))))
    }

    pub fn new(f: T) -> Self {
        Resource(RefCell::new(ResourceKind::Data(f)))
    }

    pub fn try_into(&self) -> AssetResult<T> {
        match &mut *self.0.borrow_mut() {
            &mut ResourceKind::Future(ref mut f) => {
                return match f.poll() {
                    Err(e) => Err(e),
                    Ok(Async::NotReady) => Err(AssetError::NotReady),
                    Ok(Async::Ready(i)) => Ok(i),
                };
            }

            img @ &mut ResourceKind::Data(_) => {
                let r = mem::replace(img, ResourceKind::Consumed);
                Ok(r.try_into_data().unwrap())
            }

            _ => unreachable!(),
        }
    }

    pub fn try_borrow(&self) -> AssetResult<Ref<T>> {
        let mut data = None;

        if let &mut ResourceKind::Future(ref mut f) = &mut *self.0.borrow_mut() {
            match f.poll() {
                Err(e) => return Err(e),
                Ok(Async::NotReady) => return Err(AssetError::NotReady),
                Ok(Async::Ready(i)) => {
                    data = Some(i);
                }
            }
        }

        if let Some(i) = data {
            let kind: &mut ResourceKind<T> = &mut self.0.borrow_mut();
            mem::replace(kind, ResourceKind::Data(i));
        }

        let b0 = self.0.borrow();
        return Ok(Ref::map(b0, |t| t.try_as_data().unwrap()));
    }
}

impl<T: Debug + loader::Loadable> From<T> for Resource<T> {
    fn from(r: T) -> Resource<T> {
        Resource::new(r)
    }
}
