use std::rc::Rc;
use std::cell::{Ref, RefCell, RefMut};
use std::sync::Arc;
use std::ops::Deref;

use world::app_fs::AppEngine;
use engine::{AssetSystem, Camera, ClearOption, Component, ComponentBased, Engine, GameObject,
             IEngine, SceneTree};

use engine::imgui;
use world::fps::FPS;
use world::type_watcher::{ActorWatcher, TypeWatcher, TypeWatcherBuilder};
use world::Actor;
use world::processor::{IProcessorBuilder, Processor};

use uni_app::{now, App, AppConfig, AppEvent};
use std::default::Default;
use std::marker::PhantomData;

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

    processor_builders: Vec<Rc<Box<IProcessorBuilder>>>,
}

pub struct WorldBuilder<'a> {
    title: &'a str,
    size: Option<(u32, u32)>,
    shown_stats: Option<bool>,
    watcher_builder: TypeWatcherBuilder,
    processor_builders: Vec<Rc<Box<IProcessorBuilder>>>,
}

impl<'a> WorldBuilder<'a> {
    pub fn new(title: &str) -> WorldBuilder {
        WorldBuilder {
            title: title,
            size: None,
            shown_stats: None,
            watcher_builder: TypeWatcherBuilder::new(),
            processor_builders: Vec::new(),
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
        self.watcher_builder = self.watcher_builder.add_watcher(ActorWatcher::<T>::new());
        self
    }

    pub fn with_processor<T: Processor + Actor + 'static>(mut self) -> WorldBuilder<'a> {
        self.watcher_builder = self.watcher_builder.add_watcher(ActorWatcher::<T>::new());
        self.processor_builders.push(Rc::new(T::new_builder()));
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

        let processor_watchers = self.processor_builders
            .iter()
            .flat_map(|builder| builder.new_watchers())
            .collect();

        let watcher = self.watcher_builder
            .add_watchers(processor_watchers)
            .add_watcher(ActorWatcher::<Box<Actor>>::new())
            .build(main_tree.clone());

        let mut w = World {
            engine,
            app: Some(app),
            main_tree: main_tree.clone(),
            watcher: Rc::new(watcher),
            shown_stats: self.shown_stats.unwrap_or(false),
            fps: FPS::new(),
            events: events,
            golist: Vec::new(),
            processor_builders: self.processor_builders.clone(),
        };

        // add all processor into the scenes
        let go = w.new_game_object();
        for builder in self.processor_builders.into_iter() {
            go.borrow_mut().add_component(builder.new_processor());
        }

        w
    }
}

pub struct ComponentBorrow<T> {
    c: Arc<Component>,
    marker: PhantomData<T>,
}

impl<T> ComponentBorrow<T> {
    fn new(c: Arc<Component>) -> ComponentBorrow<T> {
        ComponentBorrow {
            c,
            marker: Default::default(),
        }
    }
}

impl<T> Deref for ComponentBorrow<T>
where
    T: 'static,
{
    type Target = RefCell<T>;

    fn deref(&self) -> &Self::Target {
        self.c.try_as::<T>().unwrap()
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

    pub fn current_camera<'a>(&self) -> Option<ComponentBorrow<Camera>> {
        if self.engine.main_camera().is_none() {
            return None;
        }

        let c = self.engine.main_camera().unwrap().clone();

        return Some(ComponentBorrow::<Camera>::new(c));
    }

    fn pre_render(&mut self) {
        let watcher = self.watcher.clone();
        watcher.pre_render(self);
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

            let loading_files = self.engine().asset_system().loading_files();

            if loading_files.len() > 0 {
                let files: Vec<String> = loading_files
                    .into_iter()
                    .map(|s| format!("loading {} ...", s))
                    .collect();

                imgui::pivot((0.0, 0.0));
                imgui::label(
                    Native(0.0, 0.0) + Pixel(8.0, 24.0),
                    &format!("{}", files.join("\n")),
                );
            }
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

        // add all processor back
        let go = self.new_game_object();
        for builder in self.processor_builders.iter() {
            go.borrow_mut().add_component(builder.new_processor());
        }
    }

    pub fn event_loop(mut self) {
        let app = self.app.take().unwrap();

        app.run(move |_app: &mut App| {
            self.engine.begin();

            self.step();

            self.pre_render();

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

    pub fn find_component<T>(&mut self) -> Option<ComponentBorrow<T>>
    where
        T: 'static + ComponentBased,
    {
        self.engine
            .find_component::<T>()
            .map(|c| ComponentBorrow::new(c))
    }
}
