use engine::{Component, ComponentBased, IntoComponentPtr, GameObject};
use world::{Handle, World};

pub trait Actor {
    // Called before first update call
    fn start_rc(&mut self, go: Handle<GameObject>, world: &mut World) {
        self.start(&mut go.borrow_mut(), world)
    }

    // Called before first update call, with GameObject itself
    fn start(&mut self, &mut GameObject, &mut World) {}

    fn update_rc(&mut self, go: Handle<GameObject>, world: &mut World) {
        self.update(&mut go.borrow_mut(), world)
    }

    fn update(&mut self, &mut GameObject, &mut World) {}

    fn new_actor<T: Actor>(t: T) -> Box<Actor>
    where
        Self: Sized,
        T: 'static,
    {
        Box::new(t)
    }
}

impl ComponentBased for Box<Actor> {}

impl IntoComponentPtr for Box<Actor> {
    fn into_component_ptr(self) -> ::std::sync::Arc<Component> {
        ::unrust::engine::Component::new(self)
    }
}