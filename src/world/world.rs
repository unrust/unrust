use std::cell::{Ref, RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use engine::{
    AssetSystem, Camera, ClearOption, Component, ComponentBased, ComponentType, Engine, GameObject,
    IEngine, SceneTree,
};
use world::app_fs::AppEngine;

use engine::imgui;
use engine::SoundSystem;
use world::fps::FPS;
use world::processor::{IProcessorBuilder, Processor};
use world::type_watcher::{ActorWatcher, TypeWatcher, TypeWatcherBuilder};
use world::Actor;

use std::default::Default;
use std::marker::PhantomData;
use uni_app::{now, App, AppConfig, AppEvent};
use uni_pad as pad;

pub type Handle<T> = Rc<RefCell<T>>;

pub struct World {
    pub sound: SoundSystem,

    app_ref: Option<&'static mut App>,

    main_tree: Rc<SceneTree>,
    fps: FPS,
    watcher: Rc<TypeWatcher>,
    shown_stats: bool,
    events: Rc<RefCell<Vec<AppEvent>>>,
    golist: Vec<Handle<GameObject>>,
    processor_builders: Vec<Rc<Box<IProcessorBuilder>>>,

    engine: AppEngine,

    // App should be the last object to drop
    app_instance: Option<App>,
}

pub struct WorldBuilder<'a> {
    title: &'a str,
    size: Option<(u32, u32)>,
    headless: bool,
    fullscreen: bool,
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
            headless: false,
            fullscreen: false,
            watcher_builder: TypeWatcherBuilder::new(),
            processor_builders: Vec::new(),
        }
    }

    pub fn with_fullscreen(mut self, b: bool) -> WorldBuilder<'a> {
        self.fullscreen = b;
        self
    }

    pub fn with_headless(mut self, b: bool) -> WorldBuilder<'a> {
        self.headless = b;
        self
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
        let pb = T::new_builder();
        self.watcher_builder = pb.register_watchers(self.watcher_builder);
        self.processor_builders.push(Rc::new(pb));
        self
    }

    pub fn build<'b>(self) -> World {
        let size = self.size.unwrap_or((800, 600));
        let mut config = AppConfig::new(self.title, size);
        config.headless = self.headless;
        config.fullscreen = self.fullscreen;

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

        pad::gamepad_init();

        let watcher = self.watcher_builder
            .add_watcher(ActorWatcher::<Box<Actor>>::new())
            .build(main_tree.clone());

        let asys = engine.asset_system.clone();

        let mut w = World {
            sound: SoundSystem::new(asys),
            engine,
            app_instance: Some(app),
            main_tree: main_tree.clone(),
            watcher: Rc::new(watcher),
            shown_stats: self.shown_stats.unwrap_or(false),
            fps: FPS::new(),
            events: events,
            golist: Vec::new(),
            processor_builders: self.processor_builders.clone(),
            app_ref: None,
        };

        // add all processor into the scenes
        let go = w.new_game_object();
        for builder in self.processor_builders.into_iter() {
            go.borrow_mut()
                .add_component(builder.new_processor(&w.engine.arena));
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
    type Target = ComponentType<T>;

    fn deref(&self) -> &Self::Target {
        self.c.try_as::<T>().unwrap()
    }
}

#[cfg(not(feature = "flame_it"))]
mod profile {
    use super::*;
    pub fn dump(_evt: &AppEvent) {}

    pub fn clear() {}
}

#[cfg(feature = "flame_it")]
mod profile {
    use super::*;
    use std::cell::Cell;

    thread_local!(static NEED_DUMP: Cell<bool> = Cell::new(false););

    pub fn dump(evt: &AppEvent) {
        if let &AppEvent::KeyUp(ref k) = evt {
            if k.ctrl && k.code == "KeyP" {
                NEED_DUMP.with(|flag| {
                    flag.set(true);
                });
            }
        }
    }

    pub fn clear() {
        use flame;
        NEED_DUMP.with(|flag| {
            if flag.get() {
                use flame;
                use std::fs::File;

                flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap();
                println!("flame-graph.html was dumped.");

                flag.set(false);
            }
        });

        flame::clear();
    }
}

impl<'a> World {
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

    pub fn engine_mut(&mut self) -> &mut AppEngine {
        &mut self.engine
    }

    pub fn current_camera(&self) -> Option<ComponentBorrow<Camera>> {
        if self.engine.main_camera().is_none() {
            return None;
        }

        let c = self.engine.main_camera().unwrap().clone();

        return Some(ComponentBorrow::<Camera>::new(c));
    }

    #[cfg_attr(feature = "flame_it", flame)]
    fn pre_render(&mut self) {
        let watcher = self.watcher.clone();
        watcher.pre_render(self);
    }

    pub fn delta_time(&self) -> f64 {
        self.fps.delta_time()
    }

    #[cfg_attr(feature = "flame_it", flame)]
    fn step(&mut self) {
        for evt in self.events.borrow().iter() {
            match evt {
                &AppEvent::Resized(size) => self.engine.resize(size),
                _ => (),
            }

            profile::dump(evt);
        }

        let watcher = self.watcher.clone();
        watcher.step(self);

        self.sound.step();

        use engine::imgui::Metric::*;

        self.fps.step();

        if self.shown_stats {
            let loading_files = self.engine().asset_system().loading_files();

            let mut loading_stats = "".to_string();
            if loading_files.len() > 0 {
                let files: Vec<String> = loading_files
                    .into_iter()
                    .map(|s| format!("loading {} ...", s))
                    .collect();

                loading_stats = format!("{}", files.join("\n"));
            }

            imgui::pivot((0.0, 0.0));
            imgui::label(
                Native(0.0, 0.0) + Pixel(8.0, 8.0),
                &format!(
                    "fps: {} dt: {:04.2}[{:04.2}|{:04.2}-{:04.2}]ms\nnobj: {} actors:{} gobjs:{} sf:{} oc:[{}:{}] tc:[{}:{}]\n{}",
                    self.fps.fps,
                    self.fps.delta_time() * 1000.0,
                    self.fps.delta_time_stats().dt_avg * 1000.0,
                    self.fps.delta_time_stats().dt_min * 1000.0,
                    self.fps.delta_time_stats().dt_max * 1000.0,
                    self.engine().objects.len(),
                    self.watcher.len(),
                    self.main_tree.len(),
                    self.engine().stats.surfaces_count, 
                    self.engine().stats.opaque_count,self.engine().stats.total_opaque_count,
                    self.engine().stats.transparent_count, self.engine().stats.total_transparent_count,
                    loading_stats
                ),
            );
        }
    }

    pub fn events(&self) -> Ref<Vec<AppEvent>> {
        self.events.borrow()
    }

    pub fn asset_system<'b>(&'b self) -> &'b AssetSystem {
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
            go.borrow_mut()
                .add_component(builder.new_processor(&self.engine.arena));
        }
    }

    #[cfg_attr(feature = "flame_it", flame)]
    fn begin(&mut self) {
        self.engine.begin();
    }

    #[cfg_attr(feature = "flame_it", flame)]
    fn end(&mut self) {
        self.engine.end();
    }

    #[cfg_attr(feature = "flame_it", flame)]
    fn render(&mut self) {
        self.engine.render(ClearOption::default());
    }

    pub fn run_frame<'b: 'a>(&mut self, app: *mut App) {
        // We can make sure the lifetime of the App will longer then engine itself
        self.app_ref = Some(unsafe { &mut *app });

        self.begin();
        self.step();
        self.pre_render();
        self.render();
        self.end();

        profile::clear();

        self.app_ref = None;
    }

    pub fn event_loop(mut self) {
        let app = { self.app_instance.take().unwrap() };

        app.run(move |app: &mut App| {
            self.run_frame(app);
        });
    }

    pub fn poll_events(&mut self) -> bool {
        let mut a = self.app_instance.take().unwrap();

        let r = a.poll_events(|app: &mut App| {
            self.run_frame(app);
        });

        self.app_instance = Some(a);

        r
    }

    pub fn new_game_object(&mut self) -> Handle<GameObject> {
        let go = self.engine.new_game_object(&self.main_tree.root());
        self.golist.push(go.clone());
        go
    }

    pub fn remove_game_object(&mut self, go: &Handle<GameObject>) {
        self.golist.retain(|x| !Rc::ptr_eq(&x, go));
    }

    pub fn find_component<T>(&mut self) -> Option<ComponentBorrow<T>>
    where
        T: 'static + ComponentBased,
    {
        self.engine
            .find_component::<T>()
            .map(|c| ComponentBorrow::new(c))
    }

    pub fn set_fullscreen(&mut self, b: bool) {
        self.app_ref.as_mut().unwrap().set_fullscreen(b);
    }
}
