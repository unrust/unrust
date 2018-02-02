use glutin;
use std::os::raw::c_void;

use AppConfig;

pub struct App {
    window: glutin::GlWindow,
    events_loop: glutin::EventsLoop,
    pub events: Vec<AppEvent>,
}

fn translate_event(e: glutin::Event) -> Option<AppEvent> {
    match e {
        MosueInput(state) if state == ElementState::Released => Some(Click),
        _ => None,
    }
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
            events: Vec::new(),
        }
    }

    pub fn window(&self) -> &glutin::GlWindow {
        &self.window
    }

    pub fn get_proc_address(&self, name: &str) -> *const c_void {
        use glutin::GlContext;
        self.window().get_proc_address(name) as *const c_void
    }

    pub fn canvas(&self) -> &isize {
        &0
    }

    pub fn run<'a, F>(&mut self, mut callback: F)
    where
        F: 'static + FnMut() -> (),
    {
        use glutin::*;
        let mut running = true;
        while running {
            self.events.clear();
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
                    events.push(translate_event(event));
                });
            }

            callback();
            self.window.swap_buffers().unwrap();
        }
    }

    pub fn add_control_caption(&mut self) {}
}
