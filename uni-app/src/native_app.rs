mod native_keycode;

use glutin;
use std::os::raw::c_void;
use glutin::{ElementState, Event, WindowEvent};
use std::cell::RefCell;
use std::rc::Rc;
use time;

use AppConfig;
use AppEvent;

use super::events;
use self::native_keycode::translate_keycode;

pub struct App {
    window: glutin::GlWindow,
    events_loop: glutin::EventsLoop,
    exiting: bool,
    pub events: Rc<RefCell<Vec<AppEvent>>>,
}

fn translate_keyevent(input: glutin::KeyboardInput) -> String {
    match input.virtual_keycode {
        Some(k) => {
            let mut s = translate_keycode(k).into();
            if s == "" {
                s = format!("{:?}", k);
            }
            s
        }
        None => "".into(),
    }
}

fn translate_event(e: glutin::Event) -> Option<AppEvent> {
    if let Event::WindowEvent {
        event: winevent, ..
    } = e
    {
        match winevent {
            WindowEvent::MouseInput { state, .. } if state == ElementState::Released => {
                Some(AppEvent::Click(events::ClickEvent {}))
            }
            WindowEvent::CursorMoved { position, .. } => Some(AppEvent::MousePos(position)),
            WindowEvent::KeyboardInput { input, .. } => match input.state {
                ElementState::Pressed => Some(AppEvent::KeyDown(events::KeyDownEvent {
                    code: translate_keyevent(input),
                    shift: input.modifiers.shift,
                    alt: input.modifiers.alt,
                    ctrl: input.modifiers.ctrl,
                })),
                ElementState::Released => Some(AppEvent::KeyUp(events::KeyUpEvent {
                    code: translate_keyevent(input),
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
        let window = glutin::WindowBuilder::new()
            .with_title(config.title)
            .with_dimensions(config.size.0, config.size.1);
        let context = glutin::ContextBuilder::new()
            .with_vsync(config.vsync)
            .with_gl(GlRequest::GlThenGles {
                opengl_version: (3, 2),
                opengles_version: (2, 0),
            })
            .with_gl_profile(GlProfile::Core);

        let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
        if !config.show_cursor {
            gl_window.set_cursor_state(CursorState::Hide).unwrap();
        }
        unsafe {
            gl_window.make_current().unwrap();
        }
        App {
            window: gl_window,
            events_loop,
            exiting: false,
            events: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn print<T: Into<String>>(msg: T) {
        print!("{}", msg.into());
    }

    pub fn hidpi_factor(&self) -> f32 {
        return self.window.hidpi_factor();
    }

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
            }

            if !running {
                break;
            }

            callback(&mut self);
            self.events.borrow_mut().clear();

            self.window.swap_buffers().unwrap();

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
