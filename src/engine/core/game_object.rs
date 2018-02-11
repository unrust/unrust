use na::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::any::{Any, TypeId};
//use std::marker::PhantomData;

use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

fn next_component_id() -> u64 {
    static CURR_COMPONENT_COUNTER: AtomicU32 = AtomicU32::new(1);;

    CURR_COMPONENT_COUNTER.fetch_add(1, Ordering::SeqCst) as u64
}

pub trait Component: Any {
    fn id(&self) -> u64;
    fn typeid(&self) -> TypeId;

    fn as_any(&self) -> &Any;
}

pub struct ComponentType<T> {
    com: Rc<RefCell<T>>,
    id: u64,
}

impl<T> Component for ComponentType<T>
where
    T: 'static,
{
    fn id(&self) -> u64 {
        self.id
    }

    fn typeid(&self) -> TypeId {
        return TypeId::of::<T>();
    }

    fn as_any(&self) -> &Any {
        self
    }
}

pub trait ComponentBased {}

impl Component {
    pub fn try_into<T>(&self) -> Option<&RefCell<T>>
    where
        T: 'static,
    {
        let a = self.as_any();
        match a.downcast_ref::<ComponentType<T>>() {
            Some(t) => Some(t.com.as_ref()),
            _ => None,
        }
    }

    pub fn new<T>(value: T) -> Arc<Component>
    where
        T: 'static,
    {
        let c = ComponentType {
            com: Rc::new(RefCell::new(value)),
            id: next_component_id(),
        };

        Arc::new(c)
    }
}

pub struct GameObject {
    pub transform: Isometry3<f32>,
    pub scale: Vector3<f32>,

    pub components: Vec<Arc<Component>>,
}

pub trait IntoComponentPtr {
    fn into_component_ptr(self) -> Arc<Component>;
}

impl<T> IntoComponentPtr for T
where
    T: ComponentBased + 'static,
{
    fn into_component_ptr(self) -> Arc<Component> {
        Component::new(self)
    }
}

impl IntoComponentPtr for Arc<Component> {
    fn into_component_ptr(self) -> Arc<Component> {
        self
    }
}

impl GameObject {
    pub fn find_component<T>(&self) -> Option<(&RefCell<T>, Arc<Component>)>
    where
        T: 'static,
    {
        let typeid = TypeId::of::<T>();

        match self.components.iter().find(|c| c.typeid() == typeid) {
            Some(c) => {
                let com: &Component = c.as_ref();
                Some((com.try_into::<T>().unwrap(), c.clone()))
            }
            _ => None,
        }
    }

    pub fn add_component<T>(&mut self, c: T) -> Arc<Component>
    where
        T: IntoComponentPtr,
    {
        let p: Arc<Component> = c.into_component_ptr();
        self.components.push(p.clone());
        p
    }
}
