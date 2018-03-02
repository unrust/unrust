use std::rc::Rc;
use std::rc;
use std::cell::{Ref, RefCell, RefMut};
use std::sync::Arc;
use std::collections::BTreeSet;

use world::app_fs::AppEngine;
use engine::{AssetSystem, Camera, ClearOption, Component, ComponentBased, Engine, GameObject,
             IEngine};
use engine::imgui;

use uni_app::{App, AppConfig, AppEvent, FPS};
use std::mem;

pub type Handle<T> = Rc<RefCell<T>>;
type WeakHandle<T> = rc::Weak<RefCell<T>>;

pub struct World {
    list: Vec<Handle<GameObject>>,
    engine: AppEngine,
    app: Option<App>,
    fps: FPS,
    root_go: rc::Weak<RefCell<GameObject>>,

    // we store a strong reference here
    actors: Vec<(WeakHandle<GameObject>, Arc<Component>)>,
    actor_ids: BTreeSet<u64>,

    shown_stats: bool,
    events: Rc<RefCell<Vec<AppEvent>>>,
}

pub struct WorldBuilder<'a> {
    title: &'a str,
    size: Option<(u32, u32)>,
    shown_stats: Option<bool>,
}

impl<'a> WorldBuilder<'a> {
    pub fn new(title: &str) -> WorldBuilder {
        WorldBuilder {
            title: title,
            size: None,
            shown_stats: None,
        }
    }

    pub fn with_size(mut self, size: (u32, u32)) -> WorldBuilder<'a> {
        self.size = Some(size);
        self
    }

    pub fn with_stats(mut self, stats: bool) -> WorldBuilder<'a> {
        self.shown_stats = Some(stats);
        self
    }

    pub fn build(self) -> World {
        let size = self.size.unwrap_or((800, 600));
        let config = AppConfig::new(self.title, size);
        let app = App::new(config);
        let engine = Engine::new(app.canvas(), size);
        let events = app.events.clone();

        let mut w = World {
            list: Vec::new(),
            engine,
            app: Some(app),
            root_go: rc::Weak::new(),
            actors: Vec::new(),
            actor_ids: BTreeSet::new(),
            shown_stats: self.shown_stats.unwrap_or(false),
            fps: FPS::new(),
            events: events,
        };

        // add a root game object
        w.root_go = Rc::downgrade(&w.new_game_object());

        // Add a default camera
        w.engine.main_camera = Some(Rc::new(RefCell::new(Camera::new())));

        w
    }
}

impl World {
    pub fn root(&self) -> RefMut<GameObject> {
        self.list[0].borrow_mut()
    }

    pub fn engine(&self) -> &AppEngine {
        &self.engine
    }

    fn collect_new_actors(
        &mut self,
        new_actors: &mut Vec<(Handle<GameObject>, Arc<Component>)>,
        node: &Handle<GameObject>,
    ) {
        let mut n = node.borrow_mut();
        if n.changed() {
            if let Some((_, c)) = n.find_component::<Box<Actor>>() {
                let id = c.id();

                if !self.actor_ids.contains(&id) {
                    self.actors.push((Rc::downgrade(node), c.clone()));
                    self.actor_ids.insert(id);
                    new_actors.push((node.clone(), c.clone()));
                }
            }

            n.clear_changed();
        }
    }

    fn active_starting_actors(&mut self) {
        // Collect dirty actors
        let mut new_actors = Vec::new();

        let mut new_list = Vec::new();
        new_list.append(&mut self.list);

        for go in new_list.iter_mut() {
            self.collect_new_actors(&mut new_actors, go);
        }
        self.list.append(&mut new_list);

        for (go, c) in new_actors.into_iter() {
            let actor = c.try_as::<Box<Actor>>().unwrap();

            (*actor).borrow_mut().start(&mut go.borrow_mut(), self);
        }
    }

    fn step(&mut self) {
        for evt in self.events.borrow().iter() {
            match evt {
                &AppEvent::Resized(size) => self.engine.resize(size),
                _ => (),
            }
        }

        self.active_starting_actors();

        let mut currents = Vec::new();
        currents.append(&mut self.actors);

        for &(ref wgo, ref c) in currents.iter() {
            if let Some(go) = wgo.upgrade() {
                let actor = c.try_as::<Box<Actor>>().unwrap();

                (*actor).borrow_mut().update(&mut go.borrow_mut(), self);
            }
        }

        // move back and remove all unused components
        currents.retain(|&(_, ref c)| {
            if Arc::strong_count(c) > 1 {
                return true;
            }
            self.actor_ids.remove(&c.id());
            false
        });
        self.actors.append(&mut currents);

        use engine::imgui::Metric::*;

        self.fps.step();

        if self.shown_stats {
            imgui::pivot((0.0, 0.0));
            imgui::label(
                Native(0.0, 0.0) + Pixel(8.0, 8.0),
                &format!(
                    "fps: {} nobj: {} r:{}",
                    self.fps.fps,
                    self.engine().objects.len(),
                    self.actors.len(),
                ),
            );
        }
    }

    pub fn events(&self) -> Ref<Vec<AppEvent>> {
        self.events.borrow()
    }

    pub fn asset_system<'a>(&'a self) -> &'a AssetSystem {
        self.engine.asset_system()
    }

    pub fn reset(&mut self) {
        self.list.clear();
        self.engine.asset_system_mut().reset();

        // add back root object
        self.root_go = Rc::downgrade(&self.new_game_object());
    }

    pub fn event_loop(mut self) {
        let app = mem::replace(&mut self.app, None).unwrap();

        app.run(move |_app: &mut App| {
            self.engine.begin();

            self.step();

            // Render
            self.engine.render(ClearOption {
                color: None,
                clear_color: true,
                clear_depth: true,
                clear_stencil: false,
            });

            // End
            self.engine.end();
        });
    }

    pub fn new_game_object(&mut self) -> Handle<GameObject> {
        let go = self.engine.new_gameobject();
        self.list.push(go.clone());

        go
    }

    pub fn remove_game_object(&mut self, go: &Handle<GameObject>) {
        self.list.retain(|ref x| !Rc::ptr_eq(&x, go));
    }
}

pub trait Actor {
    fn start(&mut self, &mut GameObject, &mut World) {}

    fn update(&mut self, &mut GameObject, &mut World) {}

    fn render(&mut self, &mut GameObject, &mut World) {}

    fn new() -> Box<Actor>
    where
        Self: Sized;
}

impl ComponentBased for Box<Actor> {}
