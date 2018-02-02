use glutin;
use std::os::raw::c_void;
use glutin::{ElementState, Event, WindowEvent};
use std::cell::RefCell;
use std::rc::Rc;
use time;

use AppConfig;
use AppEvent;

use FPS;

pub struct App {
    window: glutin::GlWindow,
    events_loop: glutin::EventsLoop,
    pub events: Rc<RefCell<Vec<AppEvent>>>,
}

fn translate_event(e: glutin::Event) -> Option<AppEvent> {
    if let Event::WindowEvent {
        event: winevent, ..
    } = e
    {
        if let WindowEvent::MouseInput { state, .. } = winevent {
            if state == ElementState::Released {
                return Some(AppEvent::Click);
            }
        }
    }

    None
}

impl App {
    pub fn new(config: AppConfig) -> App {
        use glutin::*;
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_title(config.title)
            .with_dimensions(config.size.0, config.size.1);
        let context = glutin::ContextBuilder::new().with_vsync(config.vsync);
        let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

        unsafe {
            gl_window.make_current().unwrap();
        }
        App {
            window: gl_window,
            events_loop,
            events: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn add_control_text(&self) {}

    pub fn window(&self) -> &glutin::GlWindow {
        &self.window
    }

    pub fn get_proc_address(&self, name: &str) -> *const c_void {
        use glutin::GlContext;
        self.window().get_proc_address(name) as *const c_void
    }

    pub fn canvas<'p>(&'p self) -> Box<'p + FnMut(&str) -> *const c_void> {
        Box::new(move |name| self.get_proc_address(name))
    }

    pub fn run<'a, F>(mut self, mut callback: F)
    where
        F: 'static + FnMut(&mut Self) -> (),
    {
        use glutin::*;
        let mut running = true;
        while running {
            {
                let (window, events_loop, events) =
                    (&self.window, &mut self.events_loop, &mut self.events);
                events_loop.poll_events(|event| {
                    match event {
                        glutin::Event::WindowEvent { ref event, .. } => match event {
                            &glutin::WindowEvent::Closed => running = false,
                            &glutin::WindowEvent::Resized(w, h) => window.resize(w, h),
                            _ => (),
                        },
                        _ => (),
                    };
                    translate_event(event).map(|evt| events.borrow_mut().push(evt));
                });
            }

            callback(&mut self);
            self.events.borrow_mut().clear();

            self.window.swap_buffers().unwrap();
        }
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
        return time::precise_time_s();
    }

    pub fn step(&mut self) {
        self.counter += 1;
        let curr = FPS::now();
        if curr - self.last > 1.0 {
            self.last = curr;
            self.fps = self.counter;
            self.counter = 0;
            println!("{}", self.fps)
            //     let _content = document().query_selector("#fps");
            //     js!( @{_content}.innerText = "fps : " + @{self.fps} );
        }
    }
}
