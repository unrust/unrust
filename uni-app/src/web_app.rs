use stdweb;
use AppConfig;

use stdweb::web::{Element, IEventTarget, INode};
use stdweb::web::{document, window};
use stdweb::web::event::ClickEvent;
use stdweb::unstable::TryInto;

use std::cell::RefCell;
use std::rc::Rc;

use AppEvent;

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

    pub fn add_control_text(&self) {
        let div = document().create_element("div");
        let content = document().create_text_node("Click on canvas to drop new box.");
        div.append_child(&content);

        let body = document().query_selector("body").unwrap();
        body.append_child(&div);

        js!{
            var div = @{div};
            div.id = "caption";
            div.style.position = "fixed";
            div.style.top = "580px";
            div.style.left = "5px";
            div.style.padding = "5px";
            div.style.backgroundColor = "lightblue";
            div.style.textAlign = "center";
        };
    }
}

pub struct FPS {
    counter: u32,
    last: f64,
    fps: u32,
}

impl FPS {
    pub fn new() -> FPS {
        FPS::setup_div();

        let fps = FPS {
            counter: 0,
            last: FPS::now(),
            fps: 0,
        };

        fps
    }

    pub fn setup_div() {
        let div = document().create_element("div");
        let content = document().create_text_node("None");
        div.append_child(&content);

        let body = document().query_selector("body").unwrap();
        body.append_child(&div);

        js!{
            var div = @{div};
            div.id = "fps";
            div.style.position = "fixed";
            div.style.top = "5px";
            div.style.left = "5px";
            div.style.padding = "5px";
            div.style.backgroundColor = "lightblue";
            div.style.textAlign = "center";
        };
    }

    pub fn now() -> f64 {
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
            let _content = document().query_selector("#fps");
            js!( @{_content}.innerText = "fps : " + @{self.fps} );
        }
    }
}
