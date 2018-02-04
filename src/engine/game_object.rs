use na::*;
use std::rc::Rc;
use std::sync::Arc;
use std::any::{Any, TypeId};

use Engine;

pub trait Component: Any {
    fn id(&self) -> u64;
    fn typeid(&self) -> TypeId;

    fn as_any(&self) -> &Any;
}

pub struct ComponentType<T>
where
    T: ComponentBased,
{
    com: Rc<T>,
    id: u64,
}

impl<T> Component for ComponentType<T>
where
    T: 'static + ComponentBased,
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
    fn try_into<T>(&self) -> Option<&T>
    where
        T: 'static + ComponentBased,
    {
        let a = self.as_any();
        match a.downcast_ref::<ComponentType<T>>() {
            Some(t) => Some(t.com.as_ref()),
            _ => None,
        }
    }

    pub fn new<T>(value: T) -> Arc<Component>
    where
        T: 'static + ComponentBased,
    {
        let c = ComponentType {
            com: Rc::new(value),
            id: Engine::next_component_id(),
        };

        Arc::new(c)
    }
}

pub struct GameObject {
    pub transform: Isometry3<f32>,
    pub components: Vec<Arc<Component>>,
}

impl GameObject {
    pub fn get_component_by_type<T>(&self) -> Option<(&T, &Component)>
    where
        T: 'static + ComponentBased,
    {
        let typeid = TypeId::of::<T>();

        match self.components.iter().find(|c| c.typeid() == typeid) {
            Some(c) => {
                let com: &Component = c.as_ref();
                Some((com.try_into::<T>().unwrap(), com))
            }
            _ => None,
        }
    }

    pub fn add_component(&mut self, c: Arc<Component>) {
        self.components.push(c.clone());
    }
}
