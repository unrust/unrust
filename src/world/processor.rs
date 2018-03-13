use world::{Actor, ComponentBased, GameObject, Handle, World};
use engine::{Material, Mesh};
use std::marker::PhantomData;
use world::type_watcher::{GameObjectComponentPair, NewObjectList, Watcher};

use engine::Component;
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;

struct ProcessorWatcher<T> {
    marker: PhantomData<T>,
    context: Rc<ProcessorContext>,
}

impl<T> Watcher for ProcessorWatcher<T>
where
    T: Processor + 'static,
{
    fn is(&self, c: &Arc<Component>) -> bool {
        c.try_as::<T>().is_some()
    }

    fn object_start(&self, go: &Handle<GameObject>, com: &Arc<Component>, world: &mut World) {
        let processor = com.try_as::<T>().unwrap();
        (*processor).borrow_mut().start_rc(go.clone(), world);
    }

    fn object_step(&self, go: &Handle<GameObject>, com: &Arc<Component>, world: &mut World) {
        let processor = com.try_as::<T>().unwrap();
        (*processor).borrow_mut().update_rc(go.clone(), world);
    }

    fn watch_pre_render(
        &self,
        objects: &RefCell<Vec<GameObjectComponentPair>>,
        _world: &mut World,
    ) {
        let mut processor_components = Vec::new();
        {
            for &(ref wgo, ref c) in objects.borrow().iter() {
                if let Some(go) = wgo.upgrade() {
                    processor_components.push((go.clone(), c.clone()));
                }
            }
        }

        let materials = self.context.materials.borrow().clone();

        for (_, c) in processor_components.into_iter() {
            let com = c.upgrade().unwrap().clone();
            let processor = com.try_as::<T>().unwrap();
            processor.borrow().apply_materials(&materials);
        }
    }
}

#[derive(Default)]
struct MaterialWatcher {
    context: Rc<ProcessorContext>,
}

#[derive(Default)]
pub struct ProcessorContext {
    pub materials: RefCell<Vec<Rc<Material>>>,
}

impl Watcher for MaterialWatcher {
    fn is(&self, c: &Arc<Component>) -> bool {
        c.try_as::<Mesh>().is_some()
    }

    fn watch_step(
        &self,
        new_actors: &RefCell<NewObjectList>,
        actors: &RefCell<Vec<GameObjectComponentPair>>,
        _world: &mut World,
    ) {
        actors
            .borrow_mut()
            .append(&mut new_actors.borrow_mut().list);

        let mut materials = Vec::new();
        {
            for &(_, ref c) in actors.borrow().iter() {
                if let Some(cc) = c.upgrade() {
                    let mesh = cc.try_as::<Mesh>().unwrap();

                    for surface in mesh.borrow().surfaces.iter() {
                        materials.push(surface.material.clone());
                    }
                }
            }
        }

        *self.context.materials.borrow_mut() = materials;
    }
}

pub trait Processor
where
    Self: Actor + ComponentBased,
{
    fn new_builder() -> Box<IProcessorBuilder>
    where
        Self: Sized + 'static,
    {
        Box::new(ProcessorBuilder::<Self> {
            marker: PhantomData::default(),
            context: Default::default(),
        })
    }

    fn new() -> Self
    where
        Self: Sized;

    fn apply_materials(&self, materials: &Vec<Rc<Material>>);
}

pub trait IProcessorBuilder {
    fn new_processor(&self) -> Arc<Component>;

    fn new_watchers(&self) -> Vec<Box<Watcher>>;
}

pub struct ProcessorBuilder<T> {
    marker: PhantomData<T>,
    context: Rc<ProcessorContext>,
}

impl<T> IProcessorBuilder for ProcessorBuilder<T>
where
    T: Processor + 'static,
{
    fn new_processor(&self) -> Arc<Component> {
        Component::new(T::new())
    }

    fn new_watchers(&self) -> Vec<Box<Watcher>> {
        vec![
            Box::new(ProcessorWatcher::<T> {
                marker: PhantomData::default(),
                context: self.context.clone(),
            }),
            Box::new(MaterialWatcher {
                context: self.context.clone(),
            }),
        ]
    }
}
