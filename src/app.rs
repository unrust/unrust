use stdweb;
use stdweb::Once;

use stdweb::web::Element;

pub struct AppConfig {
    pub title: String,
    pub size: (u32, u32),
    pub vsync: bool,
}

impl AppConfig {
    pub fn new<T: Into<String>>(title: T, size: (u32, u32)) -> AppConfig {
        AppConfig {
            title: title.into(),
            size,
            vsync: true,
        }
    }
}

pub struct App {
    window: Element,
}

impl App {
    pub fn new(config: AppConfig) -> App {
        use stdweb::web::*;

        let _ = stdweb::initialize();
        let canvas = document().create_element("canvas");

        js!{ (@{&canvas}).width = @{config.size.0} ; @{&canvas}.height = @{config.size.1};  };

        document()
            .query_selector("body")
            .unwrap()
            .append_child(&canvas);
        App { window: canvas }
    }

    pub fn canvas(&self) -> &stdweb::web::Element {
        &self.window
    }

    pub fn request_animation_frame<F: FnOnce(f64) + 'static>(callback: F) {
        js!{
            var callback = @{Once(callback)};
            window.requestAnimationFrame(callback);
        };
    }

    pub fn game_loop(mut cb: Box<'static + FnMut() -> ()>) {
        cb();

        App::request_animation_frame(move |_: f64| {
            App::game_loop(cb);
        });
    }

    pub fn run<'a, F>(&mut self, callback: F)
    where
        F: 'static + FnMut() -> (),
    {
        let cb = Box::new(callback);

        App::request_animation_frame(move |_: f64| {
            App::game_loop(cb);
        });

        stdweb::event_loop();
    }
}
