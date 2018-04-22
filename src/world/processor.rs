use engine::{ComponentArena, Material, Mesh};
use std::marker::PhantomData;
use world::type_watcher::{GameObjectComponentPair, TypeWatcherBuilder, Watcher};
use world::{ComponentBased, GameObject, Handle, World};

use engine::Component;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

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

    fn watch_step(&self, _objects: &Vec<(Handle<GameObject>, Arc<Component>)>, _world: &mut World) {
    }

    fn watch_pre_render(
        &self,
        objects: &RefCell<Vec<GameObjectComponentPair>>,
        _world: &mut World,
    ) {
        if T::watch_material() {
            let processor_components = objects
                .borrow()
                .iter()
                .map(|&(_, ref wc)| wc.upgrade())
                .flat_map(|x| x)
                .collect::<Vec<_>>();

            let materials = self.context.materials.borrow().clone();

            for com in processor_components.into_iter() {
                let processor = com.try_as::<T>().unwrap();
                processor.borrow().apply_materials(&materials);
            }
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

    fn watch_step(&self, objects: &Vec<(Handle<GameObject>, Arc<Component>)>, _world: &mut World) {
        let mut materials = Vec::new();
        {
            for &(_, ref c) in objects.iter() {
                let mesh = c.try_as::<Mesh>().unwrap();

                for surface in mesh.borrow().surfaces.iter() {
                    materials.push(surface.material.clone());
                }
            }
        }

        *self.context.materials.borrow_mut() = materials;
    }
}

pub trait Processor
where
    Self: ComponentBased,
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

    fn watch_material() -> bool
    where
        Self: Sized,
    {
        return false;
    }

    fn apply_materials(&self, &Vec<Rc<Material>>) {}
}

pub trait IProcessorBuilder {
    fn new_processor(&self, arena: &Rc<ComponentArena>) -> Arc<Component>;

    fn register_watchers(&self, TypeWatcherBuilder) -> TypeWatcherBuilder;
}

pub struct ProcessorBuilder<T> {
    marker: PhantomData<T>,
    context: Rc<ProcessorContext>,
}

impl<T> IProcessorBuilder for ProcessorBuilder<T>
where
    T: Processor + 'static,
{
    fn new_processor(&self, arena: &Rc<ComponentArena>) -> Arc<Component> {
        Component::new(T::new(), arena)
    }

    fn register_watchers(&self, mut builder: TypeWatcherBuilder) -> TypeWatcherBuilder {
        builder = builder.add_watcher(ProcessorWatcher::<T> {
            marker: PhantomData::default(),
            context: self.context.clone(),
        });

        if T::watch_material() {
            builder = builder.add_watcher(MaterialWatcher {
                context: self.context.clone(),
            });
        }

        builder
    }
}
