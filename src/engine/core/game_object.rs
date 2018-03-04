use std::rc::Rc;
use std::rc;
use std::cell::{Ref, RefCell, RefMut};
use std::sync::Arc;
use std::any::{Any, TypeId};
use na::{Isometry3, Matrix4};
use super::scene_tree::{ComponentEvent, NodeTransform, SceneTree};

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
    pub fn try_as<T>(&self) -> Option<&RefCell<T>>
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
        T: ComponentBased + 'static,
    {
        let c = ComponentType {
            com: Rc::new(RefCell::new(value)),
            id: next_component_id(),
        };

        Arc::new(c)
    }
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

pub struct GameObjectUtil {}

impl GameObjectUtil {
    pub fn make(node_id: u64, tree: rc::Weak<SceneTree>) -> GameObject {
        GameObject {
            transform: Transform::new(node_id, tree),
            active: true,
            components: vec![],
        }
    }

    pub fn set_tree(node_id: u64, go: &mut GameObject, tree: rc::Weak<SceneTree>) {
        go.transform.tree = tree;
        go.transform.node_id = node_id;
    }

    pub fn node_id(go: &GameObject) -> u64 {
        go.transform.node_id
    }
}

impl Drop for GameObject {
    fn drop(&mut self) {
        self.transform.tree.upgrade().map(|x| {
            self.clear_components();
            x.remove_node(self.transform.node_id)
        });
    }
}

pub struct Transform {
    node_id: u64,
    tree: rc::Weak<SceneTree>,
}

impl Transform {
    fn new(node_id: u64, tree: rc::Weak<SceneTree>) -> Transform {
        Transform { node_id, tree }
    }

    pub fn as_local_matrix(&self) -> Matrix4<f32> {
        let tree = self.tree.upgrade().unwrap();
        tree.get_local_matrix(self.node_id)
    }

    pub fn as_global_matrix(&self) -> Matrix4<f32> {
        self.parent_global_matrix() * self.as_local_matrix()
    }

    fn parent_global_matrix(&self) -> Matrix4<f32> {
        let tree = self.tree.upgrade().unwrap();
        let parent_id = tree.get_parent_id(self.node_id);

        tree.get_global_matrix(parent_id)
    }

    pub fn global(&self) -> Isometry3<f32> {
        let tree = self.tree.upgrade().unwrap();
        tree.get_global_transform(self.node_id).transform
    }

    pub fn parent_global(&self) -> NodeTransform {
        let tree = self.tree.upgrade().unwrap();
        let parent_id = tree.get_parent_id(self.node_id);
        tree.get_global_transform(parent_id)
    }

    pub fn set_global(&mut self, trans: Isometry3<f32>) {
        self.set_local(self.parent_global().transform.inverse() * trans);
    }

    pub fn local(&self) -> Isometry3<f32> {
        let tree = self.tree.upgrade().unwrap();
        let local = tree.get_local_transform(self.node_id);
        return local.transform;
    }

    pub fn set_local(&mut self, trans: Isometry3<f32>) {
        let tree = self.tree.upgrade().unwrap();
        let mut local = tree.get_local_transform(self.node_id);
        local.transform = trans;
        tree.set_local_transform(self.node_id, local);
    }
}

pub struct GameObject {
    pub transform: Transform,
    pub active: bool,
    components: Vec<Arc<Component>>,
}

impl GameObject {
    // Create an empty GameObject which cannot cannot be added in SceneRoot
    pub fn empty() -> Rc<RefCell<GameObject>> {
        Rc::new(RefCell::new(GameObject {
            transform: Transform::new(0, rc::Weak::new()),
            active: true,
            components: vec![],
        }))
    }

    pub fn tree(&self) -> Rc<SceneTree> {
        self.transform.tree.upgrade().unwrap()
    }

    pub fn find_component<T>(&self) -> Option<(Ref<T>, Arc<Component>)>
    where
        T: 'static,
    {
        let typeid = TypeId::of::<T>();

        match self.components.iter().find(|c| c.typeid() == typeid) {
            Some(c) => {
                let com: &Component = c.as_ref();
                Some((com.try_as::<T>().unwrap().borrow(), c.clone()))
            }
            _ => None,
        }
    }

    pub fn find_component_mut<T>(&self) -> Option<(RefMut<T>, Arc<Component>)>
    where
        T: 'static,
    {
        let typeid = TypeId::of::<T>();

        match self.components.iter().find(|c| c.typeid() == typeid) {
            Some(c) => {
                let com: &Component = c.as_ref();
                Some((com.try_as::<T>().unwrap().borrow_mut(), c.clone()))
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

        self.tree()
            .notifiy_component(ComponentEvent::Add, self.transform.node_id, p.clone());

        p
    }

    pub fn remove_component(&mut self, c: Arc<Component>) {
        self.components.retain(|cc| !Arc::ptr_eq(&cc, &c));

        self.tree()
            .notifiy_component(ComponentEvent::Remove, self.transform.node_id, c.clone());
    }

    pub fn clear_components(&mut self) {
        let mut coms = Vec::new();
        coms.append(&mut self.components);

        for c in coms.into_iter() {
            self.tree().notifiy_component(
                ComponentEvent::Remove,
                self.transform.node_id,
                c.clone(),
            );
        }
    }

    //Tree Operations
    pub fn add_child(&self, child: &GameObject) -> Rc<RefCell<GameObject>> {
        assert!(child.transform.node_id != 0);

        // TODO do we need to support cross tree node?
        debug_assert!(Rc::ptr_eq(&self.tree(), &child.tree()));

        self.tree()
            .add_child(self.transform.node_id, child.transform.node_id)
    }

    pub fn parent(&self) -> Option<Rc<RefCell<GameObject>>> {
        self.tree().get_parent(self.transform.node_id)
    }

    pub fn childen(&self) -> Vec<Rc<RefCell<GameObject>>> {
        self.tree().get_childen(self.transform.node_id)
    }
}
