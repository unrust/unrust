use stdweb;
use AppConfig;

use stdweb::web::{Element, IEventTarget};
use stdweb::web::{window};
use stdweb::web::event::ClickEvent;
use stdweb::unstable::TryInto;

use std::cell::RefCell;
use std::rc::Rc;

use AppEvent;
use FPS;

pub struct App {
    window: Element,
    pub events: Rc<RefCell<Vec<AppEvent>>>,
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
        App {
            window: canvas,
            events: Rc::new(RefCell::new(vec![])),
        }
    }

    pub fn canvas(&self) -> &stdweb::web::Element {
        &self.window
    }

    pub fn run_loop<F>(mut self, mut callback: F)
    where
        F: 'static + FnMut(&mut Self) -> (),
    {
        window().request_animation_frame(move |_t: f64| {
            callback(&mut self);
            self.events.borrow_mut().clear();
            self.run_loop(callback);
        });
    }

    pub fn run<F>(self, callback: F)
    where
        F: 'static + FnMut(&mut Self) -> (),
    {
        let canvas: &Element = self.canvas();
        canvas.add_event_listener({
            let events = self.events.clone();
            move |_: ClickEvent| {
                events.borrow_mut().push(AppEvent::Click);
            }
        });

        self.run_loop(callback);

        stdweb::event_loop();
    }
}

impl FPS {
    pub fn new() -> FPS {
        let fps = FPS {
            counter: 0,
            last: FPS::now(),
            fps: 0,
        };

        fps
    }

    fn now() -> f64 {
        let v = js! { return performance.now(); };
        return v.try_into().unwrap();
    }

    pub fn step(&mut self) {
        self.counter += 1;
        let curr = FPS::now();
        if curr - self.last > 1000.0 {
            self.last = curr;
            self.fps = self.counter;
            self.counter = 0;
        }
    }
}
