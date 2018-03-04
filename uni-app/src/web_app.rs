use stdweb;
use AppConfig;

use stdweb::web::IEventTarget;
use stdweb::web::window;
use stdweb::web::event::{ClickEvent, IKeyboardEvent, KeyDownEvent, KeyUpEvent, ResizeEvent};
use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::IHtmlElement;
use stdweb::traits::IEvent;

use std::cell::RefCell;
use std::rc::Rc;

use AppEvent;
use FPS;

pub struct App {
    window: CanvasElement,
    pub events: Rc<RefCell<Vec<AppEvent>>>,
}

use super::events;

macro_rules! map_event {
    ($events:expr, $x:ident,$y:ident, $ee:ident, $e:expr ) => {
        {
            let events = $events.clone();
            move |$ee: $x| {
                $ee.prevent_default();
                events.borrow_mut().push(AppEvent::$y($e));
            }
        }
    };

    ($events:expr, $x:ident,$y:ident, $e:expr ) => {
        {
            let events = $events.clone();
            move |_: $x| {
                events.borrow_mut().push(AppEvent::$y($e));
            }
        }
    };
}

impl App {
    pub fn new(config: AppConfig) -> App {
        use stdweb::web::*;

        let _ = stdweb::initialize();
        let canvas: CanvasElement = document()
            .create_element("canvas")
            .unwrap()
            .try_into()
            .unwrap();

        js!{
            (@{&canvas}).width = @{config.size.0};
            @{&canvas}.height = @{config.size.1};

            // Make it focusable
            // https://stackoverflow.com/questions/12886286/addeventlistener-for-keydown-on-canvas
            @{&canvas}.tabIndex = 1;
        };

        document()
            .query_selector("body")
            .unwrap()
            .unwrap()
            .append_child(&canvas);
        App {
            window: canvas,
            events: Rc::new(RefCell::new(vec![])),
        }
    }

    pub fn print<T: Into<String>>(msg: T) {
        js!{ console.log(@{msg.into()})};
    }

    pub fn canvas(&self) -> &CanvasElement {
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
        let canvas: &CanvasElement = self.canvas();

        canvas.add_event_listener(map_event!{
            self.events,
            ClickEvent,
            Click,
            events::ClickEvent {}
        });

        canvas.add_event_listener(map_event!{
            self.events,
            KeyDownEvent,
            KeyDown,
            e,
            events::KeyDownEvent {
                code: e.code(),
                shift: e.shift_key(),
                alt: e.alt_key(),
                ctrl: e.ctrl_key(),
            }
        });

        // canvas.add_event_listener(map_event!{
        //     self.events,
        //     KeypressEvent,
        //     KeyPress,
        //     e,
        //     events::KeyPressEvent {
        //         code: e.code()
        //     }
        // });

        canvas.add_event_listener(map_event!{
            self.events,
            KeyUpEvent,
            KeyUp,
            e,
            events::KeyUpEvent {
                code: e.code(),
                shift: e.shift_key(),
                alt: e.alt_key(),
                ctrl: e.ctrl_key(),
            }
        });

        canvas.add_event_listener({
            let canvas = canvas.clone();

            map_event!{
                self.events,
                ResizeEvent,
                Resized,
                (canvas.offset_width() as u32, canvas.offset_height() as u32)
            }
        });

        self.run_loop(callback);

        stdweb::event_loop();
    }
}

pub fn now() -> f64 {
    let v = js! { return performance.now()/1000.0; };
    return v.try_into().unwrap();
}

impl FPS {
    pub fn new() -> FPS {
        let fps = FPS {
            counter: 0,
            last: now(),
            fps: 0,
        };

        fps
    }

    pub fn step(&mut self) {
        self.counter += 1;
        let curr = now();
        if curr - self.last > 1000.0 {
            self.last = curr;
            self.fps = self.counter;
            self.counter = 0;
        }
    }
}
