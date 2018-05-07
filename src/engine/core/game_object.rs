use math::*;
use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::marker::PhantomData;
use std::rc;
use std::rc::Rc;
use std::sync::Arc;

use super::component_arena::ComponentArena;
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

pub struct ComponentType<T: 'static> {
    arena: Rc<ComponentArena>,
    id: u64,
    phantom: PhantomData<T>,

    // data is a kind of lock to do runtime borrow checking
    data: RefCell<()>,
}

impl<T: 'static> ComponentType<T> {
    pub fn borrow(&self) -> Ref<T> {
        let arena = self.arena.clone();
        Ref::map(self.data.borrow(), move |_| arena.get(self.id))
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        let arena = self.arena.clone();
        RefMut::map(self.data.borrow_mut(), move |_| arena.get_mut(self.id))
    }
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

impl<T> Drop for ComponentType<T>
where
    T: 'static,
{
    fn drop(&mut self) {
        self.arena.remove::<T>(self.id);
    }
}

pub trait ComponentBased {}

impl Component {
    pub fn try_as<T>(&self) -> Option<&ComponentType<T>>
    where
        T: 'static,
    {
        let a = self.as_any();
        match a.downcast_ref::<ComponentType<T>>() {
            Some(t) => Some(t),
            _ => None,
        }
    }

    pub fn new<T>(value: T, arena: &Rc<ComponentArena>) -> Arc<Component>
    where
        T: ComponentBased + 'static,
    {
        let id = next_component_id();
        arena.add(id, value);

        let c: ComponentType<T> = ComponentType {
            id: id,
            arena: arena.clone(),
            phantom: PhantomData::default(),
            data: RefCell::new(()),
        };

        Arc::new(c)
    }
}

pub trait IntoComponentPtr {
    fn into_component_ptr(self, area: &Rc<ComponentArena>) -> Arc<Component>;
}

impl IntoComponentPtr for Arc<Component> {
    fn into_component_ptr(self, _: &Rc<ComponentArena>) -> Arc<Component> {
        self
    }
}

pub struct GameObjectUtil {}

impl GameObjectUtil {
    pub fn make(node_id: u64, tree: rc::Weak<SceneTree>, arena: &Rc<ComponentArena>) -> GameObject {
        GameObject {
            transform: Transform::new(node_id, tree),
            arena: Rc::downgrade(arena),
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
        use math::*;

        self.set_local(
            self.parent_global()
                .transform
                .inverse_transform()
                .unwrap()
                .concat(&trans),
        );
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

    pub fn set_local_scale(&mut self, s: Vector3<f32>) {
        let tree = self.tree.upgrade().unwrap();
        let mut local = tree.get_local_transform(self.node_id);
        local.scale = s;
        tree.set_local_transform(self.node_id, local);
    }

    pub fn local_scale(&self) -> Vector3<f32> {
        let tree = self.tree.upgrade().unwrap();
        let local = tree.get_local_transform(self.node_id);
        local.scale
    }
}

pub struct GameObject {
    pub transform: Transform,
    pub active: bool,
    components: Vec<Arc<Component>>,
    arena: rc::Weak<ComponentArena>,
}

impl GameObject {
    // Create an empty GameObject which cannot cannot be added in SceneRoot
    pub fn empty() -> Rc<RefCell<GameObject>> {
        Rc::new(RefCell::new(GameObject {
            transform: Transform::new(0, rc::Weak::new()),
            active: true,
            arena: rc::Weak::new(),
            components: vec![],
        }))
    }

    pub fn tree(&self) -> Rc<SceneTree> {
        self.transform.tree.upgrade().unwrap()
    }

    pub fn find_component<T>(&self) -> Option<(Ref<T>, &Arc<Component>)>
    where
        T: 'static,
    {
        let typeid = TypeId::of::<T>();

        match self.components.iter().find(|c| c.typeid() == typeid) {
            Some(c) => {
                let com: &Component = c.as_ref();
                Some((com.try_as::<T>().unwrap().borrow(), c))
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
        let p: Arc<Component> = c.into_component_ptr(&self.arena.upgrade().unwrap());
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
