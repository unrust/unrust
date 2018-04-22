use engine::{ComponentBased, GameObject};
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
}

impl ComponentBased for Box<Actor> {}