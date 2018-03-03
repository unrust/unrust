use engine::core::GameObject;
use std::rc::{Rc, Weak};
use std::cell::{Ref, RefCell};

pub struct SceneTree {
    root: Rc<RefCell<GameObject>>,
    nodes: Vec<Weak<RefCell<GameObject>>>,
}

impl SceneTree {
    pub fn new() -> SceneTree {
        SceneTree {
            nodes: vec![],
            root: Rc::new(RefCell::new(GameObject::new(Weak::new()))),
        }
    }

    pub fn root(&self) -> Ref<GameObject> {
        self.root.borrow()
    }
}
