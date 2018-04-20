mod native_keycode;

use glutin;
use glutin::{ElementState, Event, MouseButton, WindowEvent};
use std::cell::RefCell;
use std::env;
use std::os::raw::c_void;
use std::rc::Rc;
use time;

use AppConfig;
use AppEvent;

use self::native_keycode::{translate_scan_code, translate_virtual_key};
use super::events;

enum WindowContext {
    Normal(glutin::GlWindow),
    Headless(glutin::HeadlessContext),
}

impl WindowContext {
    fn hidpi_factor(&self) -> f32 {
        match self {
            &WindowContext::Normal(ref w) => w.hidpi_factor(),
            _ => 1.0,
        }
    }

    fn window(&self) -> &glutin::GlWindow {
        match self {
            &WindowContext::Normal(ref w) => w,
            _ => unimplemented!(),
        }
    }

    fn context(&self) -> &glutin::GlContext {
        match self {
            &WindowContext::Normal(ref w) => w,
            &WindowContext::Headless(ref w) => w,
        }
    }
}

pub struct App {
    window: WindowContext,
    events_loop: glutin::EventsLoop,
    exiting: bool,
    pub events: Rc<RefCell<Vec<AppEvent>>>,
}

fn get_virtual_key(input: glutin::KeyboardInput) -> String {
    match input.virtual_keycode {
        Some(k) => {
            let mut s = translate_virtual_key(k).into();
            if s == "" {
                s = format!("{:?}", k);
            }
            s
        }
        None => "".into(),
    }
}

fn get_scan_code(input: glutin::KeyboardInput) -> String {
    translate_scan_code(input.scancode).into()
}

fn translate_event(e: glutin::Event) -> Option<AppEvent> {
    if let Event::WindowEvent {
        event: winevent, ..
    } = e
    {
        match winevent {
            WindowEvent::MouseInput { state, button, .. } => {
                let button_num = match button {
                    MouseButton::Left => 0,
                    MouseButton::Middle => 1,
                    MouseButton::Right => 2,
                    MouseButton::Other(val) => val as usize,
                };
                let event = events::MouseButtonEvent { button: button_num };
                match state {
                    ElementState::Pressed => Some(AppEvent::MouseDown(event)),
                    ElementState::Released => Some(AppEvent::MouseUp(event)),
                }
            }
            WindowEvent::CursorMoved { position, .. } => Some(AppEvent::MousePos(position)),
            WindowEvent::KeyboardInput { input, .. } => match input.state {
                ElementState::Pressed => Some(AppEvent::KeyDown(events::KeyDownEvent {
                    key: get_virtual_key(input),
                    code: get_scan_code(input),
                    shift: input.modifiers.shift,
                    alt: input.modifiers.alt,
                    ctrl: input.modifiers.ctrl,
                })),
                ElementState::Released => Some(AppEvent::KeyUp(events::KeyUpEvent {
                    key: get_virtual_key(input),
                    code: get_scan_code(input),
                    shift: input.modifiers.shift,
                    alt: input.modifiers.alt,
                    ctrl: input.modifiers.ctrl,
                })),
            },
            WindowEvent::Resized(w, h) => Some(AppEvent::Resized((w, h))),

            _ => None,
        }
    } else {
        None
    }
}

impl App {
    pub fn new(config: AppConfig) -> App {
        use glutin::*;
        let events_loop = glutin::EventsLoop::new();
        let gl_req = GlRequest::GlThenGles {
            opengl_version: (3, 2),
            opengles_version: (2, 0),
        };

        let window = if config.headless {
            let context = glutin::HeadlessRendererBuilder::new(config.size.0, config.size.1)
                .with_gl(gl_req)
                .with_gl_profile(GlProfile::Core)
                .build()
                .unwrap();

            WindowContext::Headless(context)
        } else {
            let window = glutin::WindowBuilder::new()
                .with_title(config.title)
                .with_dimensions(config.size.0, config.size.1);

            let context = glutin::ContextBuilder::new()
                .with_vsync(config.vsync)
                .with_gl(gl_req)
                .with_gl_profile(GlProfile::Core);

            let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
            if !config.show_cursor {
                gl_window.set_cursor_state(CursorState::Hide).unwrap();
            }

            WindowContext::Normal(gl_window)
        };

        unsafe {
            window.context().make_current().unwrap();
        }

        App {
            window: window,
            events_loop,
            exiting: false,
            events: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn get_params() -> Vec<String> {
        let mut params: Vec<String> = env::args().collect();
        params.remove(0);
        params
    }

    pub fn print<T: Into<String>>(msg: T) {
        print!("{}", msg.into());
    }

    pub fn hidpi_factor(&self) -> f32 {
        return self.window.hidpi_factor();
    }

    pub fn window(&self) -> &glutin::GlWindow {
        &self.window.window()
    }

    pub fn get_proc_address(&self, name: &str) -> *const c_void {
        self.window.context().get_proc_address(name) as *const c_void
    }

    pub fn canvas<'p>(&'p self) -> Box<'p + FnMut(&str) -> *const c_void> {
        Box::new(move |name| self.get_proc_address(name))
    }

    fn handle_events(&mut self) -> bool {
        use glutin::*;
        let mut running = true;

        let (window, events_loop, events) = (&self.window, &mut self.events_loop, &mut self.events);

        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { ref event, .. } => match event {
                    &glutin::WindowEvent::Closed => running = false,
                    &glutin::WindowEvent::Resized(w, h) => window.context().resize(w, h),
                    &glutin::WindowEvent::KeyboardInput { input, .. } => {
                        // issue tracked in https://github.com/tomaka/winit/issues/41
                        // Right now we handle it manually.
                        if cfg!(target_os = "macos") {
                            if let Some(keycode) = input.virtual_keycode {
                                if keycode == VirtualKeyCode::Q && input.modifiers.logo {
                                    running = false;
                                }
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            };
            translate_event(event).map(|evt| events.borrow_mut().push(evt));
        });

        return running;
    }

    pub fn poll_events<F>(&mut self, callback: F) -> bool
    where
        F: FnOnce(&mut Self) -> (),
    {
        if !self.handle_events() {
            return false;
        }

        callback(self);
        self.events.borrow_mut().clear();
        self.window.context().swap_buffers().unwrap();

        return !self.exiting;
    }

    pub fn run<'a, F>(mut self, mut callback: F)
    where
        F: 'static + FnMut(&mut Self) -> (),
    {
        let mut running = true;

        while running {
            running = self.handle_events();

            if !running {
                break;
            }

            callback(&mut self);
            self.events.borrow_mut().clear();
            self.window.context().swap_buffers().unwrap();

            if self.exiting {
                break;
            }
        }
    }
}

pub fn now() -> f64 {
    // precise_time_s() is in second
    // https://doc.rust-lang.org/time/time/fn.precise_time_s.html
    return time::precise_time_s();
}
