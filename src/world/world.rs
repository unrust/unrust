use std::rc::Rc;
use std::cell::{Ref, RefCell, RefMut};
use std::sync::Arc;
use std::ops::Deref;

use world::app_fs::AppEngine;
use engine::{AssetSystem, Camera, ClearOption, Component, ComponentBased, Engine, GameObject,
             IEngine, SceneTree};

use engine::imgui;
use world::fps::FPS;
use world::type_watcher::{ActorWatcher, TypeWatcher, TypeWatcherBuilder, Watcher};
use world::Actor;

use uni_app::{now, App, AppConfig, AppEvent};
use std::default::Default;

pub type Handle<T> = Rc<RefCell<T>>;

pub struct World {
    engine: AppEngine,
    main_tree: Rc<SceneTree>,

    app: Option<App>,
    fps: FPS,

    watcher: Rc<TypeWatcher>,

    shown_stats: bool,
    events: Rc<RefCell<Vec<AppEvent>>>,

    golist: Vec<Handle<GameObject>>,
}

pub struct WorldBuilder<'a> {
    title: &'a str,
    size: Option<(u32, u32)>,
    shown_stats: Option<bool>,
    watchers: Vec<Box<Watcher>>,
}

impl<'a> WorldBuilder<'a> {
    pub fn new(title: &str) -> WorldBuilder {
        WorldBuilder {
            title: title,
            size: None,
            shown_stats: None,
            watchers: Vec::new(),
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

    pub fn with_actor<T: Actor + 'static>(mut self) -> WorldBuilder<'a> {
        self.watchers.push(Box::new(ActorWatcher::<T>::new()));
        self
    }

    pub fn build(self) -> World {
        let size = self.size.unwrap_or((800, 600));
        let config = AppConfig::new(self.title, size);
        let app = App::new(config);

        let hidpi = app.hidpi_factor();
        let engine = Engine::new(
            app.canvas(),
            (
                ((size.0 as f32) * hidpi) as u32,
                ((size.1 as f32) * hidpi) as u32,
            ),
            hidpi,
        );
        let events = app.events.clone();
        let main_tree = engine.new_scene_tree();
        let watcher = TypeWatcherBuilder::new(main_tree.clone())
            .add_watcher(ActorWatcher::<Box<Actor>>::new())
            .add_watchers(self.watchers)
            .build();

        let w = World {
            engine,
            app: Some(app),
            main_tree: main_tree.clone(),
            watcher: Rc::new(watcher),
            shown_stats: self.shown_stats.unwrap_or(false),
            fps: FPS::new(),
            events: events,
            golist: Vec::new(),
        };

        w
    }
}

pub struct CameraBorrow(Arc<Component>);

impl Deref for CameraBorrow {
    type Target = RefCell<Camera>;

    fn deref(&self) -> &Self::Target {
        self.0.try_as::<Camera>().unwrap()
    }
}

impl World {
    pub fn root(&self) -> Ref<GameObject> {
        self.main_tree.root()
    }

    pub fn root_mut(&self) -> RefMut<GameObject> {
        self.main_tree.root_mut()
    }

    pub fn now() -> f64 {
        now()
    }

    pub fn engine(&self) -> &AppEngine {
        &self.engine
    }

    pub fn current_camera<'a>(&self) -> Option<CameraBorrow> {
        if self.engine.main_camera().is_none() {
            return None;
        }

        let c = self.engine.main_camera().unwrap().clone();

        return Some(CameraBorrow(c));
    }

    fn step(&mut self) {
        for evt in self.events.borrow().iter() {
            match evt {
                &AppEvent::Resized(size) => self.engine.resize(size),
                _ => (),
            }
        }

        let watcher = self.watcher.clone();
        watcher.step(self);

        use engine::imgui::Metric::*;

        self.fps.step();

        if self.shown_stats {
            imgui::pivot((0.0, 0.0));
            imgui::label(
                Native(0.0, 0.0) + Pixel(8.0, 8.0),
                &format!(
                    "fps: {} nobj: {} actors:{} lists:{}",
                    self.fps.fps,
                    self.engine().objects.len(),
                    self.watcher.len(),
                    self.main_tree.len(),
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
        self.watcher.clear();
        self.golist.clear();
        self.engine.asset_system_mut().reset();
        self.main_tree.root_mut().clear_components();
    }

    pub fn event_loop(mut self) {
        let app = self.app.take().unwrap();

        app.run(move |_app: &mut App| {
            self.engine.begin();

            self.step();

            // Render
            self.engine.render(ClearOption::default());

            // End
            self.engine.end();
        });
    }

    pub fn new_game_object(&mut self) -> Handle<GameObject> {
        let go = self.engine.new_game_object(&self.main_tree.root());
        self.golist.push(go.clone());
        go
    }

    pub fn remove_game_object(&mut self, go: &Handle<GameObject>) {
        self.golist.retain(|ref x| !Rc::ptr_eq(&x, go));
    }

    pub fn find_component<T>(&mut self) -> Option<(Arc<Component>)>
    where
        T: 'static + ComponentBased,
    {
        self.engine.find_component::<T>()
    }
}
