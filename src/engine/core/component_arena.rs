use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use typed_arena::Arena;

struct ComponentContainer<T> {
    components: Arena<T>,
    com_map: RefCell<HashMap<u64, *mut T>>,

    free_list: RefCell<Vec<*mut T>>,
}

impl<T> ComponentContainer<T> {
    fn new() -> ComponentContainer<T> {
        ComponentContainer {
            components: Arena::new(),
            com_map: RefCell::new(HashMap::new()),
            free_list: RefCell::new(Vec::new()),
        }
    }

    fn add(&self, id: u64, c: T) {
        let mut free_list = self.free_list.borrow_mut();

        let mt = if free_list.len() > 0 {
            let mt = free_list.pop().unwrap();
            unsafe {
                *mt = c;
                &mut *mt
            }
        } else {
            self.components.alloc(c)
        };

        self.com_map.borrow_mut().insert(id, mt);
    }

    fn remove(&self, id: u64) {
        let mut free_list = self.free_list.borrow_mut();

        let mt = self.com_map.borrow_mut().remove(&id).unwrap();
        free_list.push(mt);
    }

    fn get<'a>(&self, id: u64) -> &'a T {
        let p = *self.com_map.borrow().get(&id).unwrap();

        unsafe { &*p }
    }

    fn get_mut<'a>(&self, id: u64) -> &'a mut T {
        let p = *self.com_map.borrow().get(&id).unwrap();

        unsafe { &mut *p }
    }
}

pub struct ComponentArena {
    arenas: RefCell<HashMap<TypeId, Box<Any>>>,
}

impl ComponentArena {
    pub fn add<T>(&self, id: u64, c: T)
    where
        T: 'static,
    {
        self.container().add(id, c);
    }

    pub fn remove<T: 'static>(&self, id: u64)
    where
        T: 'static,
    {
        self.container::<T>().remove(id);
    }

    fn container<T: 'static>(&self) -> Ref<ComponentContainer<T>> {
        let typeid = TypeId::of::<T>();

        let mut arenas = self.arenas.borrow_mut();
        if !arenas.get(&typeid).is_some() {
            arenas.insert(typeid, Box::new(ComponentContainer::<T>::new()));
        }
        drop(arenas);

        let arenas = self.arenas.borrow();

        Ref::map(arenas, |a| {
            let container = a.get(&typeid).unwrap();

            container.downcast_ref::<ComponentContainer<T>>().unwrap()
        })
    }

    pub fn get<'b, T: 'static>(&self, id: u64) -> &'b T {
        self.container().get(id)
    }

    pub fn get_mut<'b, T: 'static>(&self, id: u64) -> &'b mut T {
        self.container().get_mut(id)
    }

    pub fn new() -> ComponentArena {
        ComponentArena {
            arenas: Default::default(),
        }
    }
}
