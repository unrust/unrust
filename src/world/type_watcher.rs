use std::rc::Rc;
use std::rc;
use std::cell::RefCell;
use std::sync;
use std::sync::Arc;
use std::marker::PhantomData;

use engine::{Component, ComponentEvent, GameObject, SceneTree};
use world::{Actor, Handle, World};

type WeakHandle<T> = rc::Weak<RefCell<T>>;
pub type GameObjectComponentPair = (WeakHandle<GameObject>, sync::Weak<Component>);

#[derive(Default)]
pub struct NewObjectList {
    pub list: Vec<GameObjectComponentPair>,
}

#[derive(Default)]
pub struct ObjectContainer {
    new_objects: RefCell<NewObjectList>,
    objects: RefCell<Vec<GameObjectComponentPair>>,
}

pub trait Watcher {
    fn is(&self, c: &Arc<Component>) -> bool;

    fn object_start(&self, _go: &Handle<GameObject>, _com: &Arc<Component>, &mut World) {}

    fn object_step(&self, _go: &Handle<GameObject>, _com: &Arc<Component>, &mut World) {}

    fn watch_pre_render(
        &self,
        _actors: &RefCell<Vec<GameObjectComponentPair>>,
        _world: &mut World,
    ) {
    }

    fn watch_step(
        &self,
        new_actors: &RefCell<NewObjectList>,
        actors: &RefCell<Vec<GameObjectComponentPair>>,
        world: &mut World,
    ) {
        while new_actors.borrow().list.len() > 0 {
            let mut starting = Vec::new();
            starting.append(&mut new_actors.borrow_mut().list);

            for &(ref wgo, ref c) in starting.iter() {
                let com = c.upgrade().unwrap().clone();
                let go = wgo.upgrade().unwrap();

                self.object_start(&go, &com, world);
            }

            actors.borrow_mut().append(&mut starting);
        }

        let mut actor_components = Vec::new();
        {
            for &(ref wgo, ref c) in actors.borrow().iter() {
                if let Some(go) = wgo.upgrade() {
                    actor_components.push((go.clone(), c.clone()));
                }
            }
        }

        for (go, c) in actor_components.into_iter() {
            let com = c.upgrade().unwrap().clone();

            self.object_step(&go, &com, world);
        }
    }
}

pub struct TypeWatcher {
    object_containers: Rc<Vec<(Box<Watcher>, ObjectContainer)>>,
}

pub struct ActorWatcher<T> {
    marker: PhantomData<T>,
}

impl<T> ActorWatcher<T> {
    pub fn new() -> ActorWatcher<T> {
        ActorWatcher {
            marker: Default::default(),
        }
    }
}

impl<T> Watcher for ActorWatcher<T>
where
    T: Actor + 'static,
{
    fn is(&self, c: &Arc<Component>) -> bool {
        c.try_as::<T>().is_some()
    }

    fn object_start(&self, go: &Handle<GameObject>, com: &Arc<Component>, world: &mut World) {
        let actor = com.try_as::<T>().unwrap();
        (*actor).borrow_mut().start_rc(go.clone(), world);
    }

    fn object_step(&self, go: &Handle<GameObject>, com: &Arc<Component>, world: &mut World) {
        let actor = com.try_as::<T>().unwrap();
        (*actor).borrow_mut().update_rc(go.clone(), world);
    }
}

impl Watcher for ActorWatcher<Box<Actor>> {
    fn is(&self, c: &Arc<Component>) -> bool {
        c.try_as::<Box<Actor>>().is_some()
    }

    fn object_start(&self, go: &Handle<GameObject>, com: &Arc<Component>, world: &mut World) {
        let actor = com.try_as::<Box<Actor>>().unwrap();
        (*actor).borrow_mut().start_rc(go.clone(), world);
    }

    fn object_step(&self, go: &Handle<GameObject>, com: &Arc<Component>, world: &mut World) {
        let actor = com.try_as::<Box<Actor>>().unwrap();
        (*actor).borrow_mut().update_rc(go.clone(), world);
    }
}

pub struct TypeWatcherBuilder {
    tree: Rc<SceneTree>,
    object_containers: Vec<(Box<Watcher>, ObjectContainer)>,
}

impl TypeWatcherBuilder {
    pub fn new(main_tree: Rc<SceneTree>) -> TypeWatcherBuilder {
        TypeWatcherBuilder {
            tree: main_tree,
            object_containers: Default::default(),
        }
    }

    pub fn add_watcher<T: Watcher + 'static>(mut self, watcher: T) -> TypeWatcherBuilder {
        self.object_containers
            .push((Box::new(watcher), ObjectContainer::default()));
        self
    }

    pub fn add_watchers(mut self, watchers: Vec<Box<Watcher>>) -> TypeWatcherBuilder {
        for watcher in watchers.into_iter() {
            self.object_containers
                .push((watcher, ObjectContainer::default()));
        }

        self
    }

    pub fn build(self) -> TypeWatcher {
        let tw = TypeWatcher {
            object_containers: Rc::new(self.object_containers),
        };

        tw.watch(self.tree)
    }
}

impl TypeWatcher {
    pub fn step(&self, world: &mut World) {
        for &(ref watcher, ref container) in self.object_containers.iter() {
            watcher.watch_step(&container.new_objects, &container.objects, world);

            // remove unused
            container
                .objects
                .borrow_mut()
                .retain(|&(_, ref c)| c.upgrade().is_some());
        }
    }

    pub fn pre_render(&self, world: &mut World) {
        for &(ref watcher, ref container) in self.object_containers.iter() {
            watcher.watch_pre_render(&container.objects, world);
        }
    }

    fn watch(self, main_tree: Rc<SceneTree>) -> Self {
        main_tree.add_watcher({
            let object_containers = self.object_containers.clone();

            move |changed, ref go, ref c: &Arc<Component>| {
                for &(ref watcher, ref container) in object_containers.iter() {
                    if watcher.is(c) {
                        match changed {
                            ComponentEvent::Add => {
                                // filter
                                let mut objects = container.new_objects.borrow_mut();
                                objects.list.push((Rc::downgrade(go), Arc::downgrade(c)));
                            }

                            ComponentEvent::Remove => {
                                let mut curr_objects = container.objects.borrow_mut();
                                curr_objects.retain(|&(_, ref cc)| {
                                    cc.upgrade().map_or(true, |ref ccp| !Arc::ptr_eq(ccp, &c))
                                });
                            }
                        }
                    }
                }
            }
        });

        self
    }

    pub fn clear(&self) {
        for &(_, ref container) in self.object_containers.iter() {
            container.objects.borrow_mut().clear();
        }
    }

    pub fn len(&self) -> usize {
        let mut n: usize = 0;

        for &(_, ref container) in self.object_containers.iter() {
            n += container.objects.borrow().len()
        }

        n
    }
}
