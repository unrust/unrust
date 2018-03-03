use engine::core::{Component, GameObject};
use std::rc::{Rc, Weak};
use std::cell::{Cell, Ref, RefCell, RefMut};
use super::internal::GameObjectUtil;
use std::collections::BTreeMap;
use std::sync::Arc;

struct Node {
    parent: u64,
    children: Vec<u64>,
    go: Weak<RefCell<GameObject>>,
}

#[derive(Copy, Clone, Debug)]
pub enum ComponentEvent {
    Add,
    Remove,
}

pub struct SceneTree {
    root: Rc<RefCell<GameObject>>,
    nodes: RefCell<BTreeMap<u64, Node>>,
    curr_id: Cell<u64>,
    weak_self: RefCell<Weak<SceneTree>>,

    component_watcher:
        RefCell<Vec<Box<FnMut(ComponentEvent, &Rc<RefCell<GameObject>>, &Arc<Component>)>>>,
}

impl SceneTree {
    pub fn add_watcher<F>(&self, f: F)
    where
        F: FnMut(ComponentEvent, &Rc<RefCell<GameObject>>, &Arc<Component>) + 'static,
    {
        self.component_watcher.borrow_mut().push(Box::new(f));
    }

    pub fn new() -> Rc<SceneTree> {
        let s = SceneTree {
            nodes: RefCell::new(BTreeMap::default()),
            root: GameObject::empty(),
            weak_self: RefCell::new(Weak::new()),
            curr_id: Cell::new(1),
            component_watcher: Default::default(),
        };

        let root = s.root.clone();

        s.nodes.borrow_mut().insert(
            0,
            Node {
                parent: 0,
                children: Vec::new(),
                go: Rc::downgrade(&root),
            },
        );

        let p = Rc::new(s);
        let weakp = Rc::downgrade(&p);
        *p.weak_self.borrow_mut() = weakp.clone();

        GameObjectUtil::set_tree(0, &mut p.root.borrow_mut(), Rc::downgrade(&p));
        p
    }

    pub fn root(&self) -> Ref<GameObject> {
        self.root.borrow()
    }

    pub fn root_mut(&self) -> RefMut<GameObject> {
        self.root.borrow_mut()
    }

    pub fn new_node(&self, parent_go: &GameObject) -> Rc<RefCell<GameObject>> {
        debug_assert!(self.weak_self.borrow().upgrade().is_some());

        let id = self.curr_id.get();
        let mut nodes = self.nodes.borrow_mut();

        let go = Rc::new(RefCell::new(GameObjectUtil::make(
            id,
            self.weak_self.borrow().clone(),
        )));

        self.curr_id.set(id + 1);
        let parent_id = GameObjectUtil::node_id(parent_go);

        // Not root
        let parent_node = nodes.get_mut(&parent_id).unwrap();
        parent_node.children.push(id);

        nodes.insert(
            id,
            Node {
                parent: parent_id,
                children: Vec::new(),
                go: Rc::downgrade(&go),
            },
        );

        go
    }

    pub fn remove_node(&self, node_id: u64) {
        let mut nodes = self.nodes.borrow_mut();
        let node = nodes.get_mut(&node_id).unwrap();

        // remove parent's children
        let parent_id = node.parent;
        let children_id = node.children.clone();
        drop(node);
        nodes.remove(&node_id);

        let parent_node = nodes.get_mut(&parent_id).unwrap();
        parent_node.children.retain(|&x| x != node_id);
        drop(parent_node);

        for child_id in children_id {
            let child_node = nodes.get_mut(&child_id).unwrap();
            // Root adapted.
            child_node.parent = 0;
        }
    }

    pub fn add_child(&self, parent_id: u64, child_id: u64) -> Rc<RefCell<GameObject>> {
        debug_assert!(child_id != 0);

        let mut nodes = self.nodes.borrow_mut();

        let child_node = nodes.get_mut(&child_id).unwrap();
        let old_parent_id = child_node.parent;
        child_node.parent = parent_id;
        drop(child_node);

        let parent_node = nodes.get_mut(&parent_id).unwrap();
        parent_node.children.push(child_id);

        let parent_node = nodes.get_mut(&old_parent_id).unwrap();
        parent_node.children.retain(|&x| x != child_id);

        parent_node.go.upgrade().unwrap_or(self.root.clone())
    }

    pub fn get_parent(&self, node_id: u64) -> Option<Rc<RefCell<GameObject>>> {
        if node_id == 0 {
            return None;
        }

        let nodes = self.nodes.borrow();
        let parent_id = nodes.get(&node_id).unwrap().parent;

        let wgo = nodes.get(&parent_id).unwrap().go.clone();

        wgo.upgrade().map(|go| go.clone())
    }

    pub fn get_childen(&self, node_id: u64) -> Vec<Rc<RefCell<GameObject>>> {
        let nodes = self.nodes.borrow();

        let node = nodes.get(&node_id).unwrap();

        node.children
            .iter()
            .filter_map(|id| nodes.get(id).unwrap().go.upgrade())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.nodes.borrow().len()
    }

    pub fn notifiy_component(&self, evt: ComponentEvent, node_id: u64, c: Arc<Component>) {
        let go = { self.nodes.borrow().get(&node_id).unwrap().go.clone() };

        let mut watchers = self.component_watcher.borrow_mut();

        for w in watchers.iter_mut() {
            go.upgrade().map(|go| {
                w(evt, &go, &c);
            });
        }
    }
}
