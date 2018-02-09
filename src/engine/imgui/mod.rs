mod widgets;

use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

use engine::core::GameObject;
use engine::IEngine;
use std::collections::HashMap;

#[derive(Default, Debug)]
struct ImguiRaw {
    id: u32,
    render_list: Vec<Arc<widgets::Widget>>,
}

struct Imgui {
    inner: Arc<Mutex<ImguiRaw>>,
}

lazy_static! {
    static ref INSTANCE: Arc<Mutex<ImguiRaw>> = {
        Arc::new(Mutex::new(Default::default()))
    };
}

fn imgui_inst() -> Imgui {
    return Imgui {
        inner: INSTANCE.clone(),
    };
}

pub fn begin() {
    let imgui = imgui_inst();
    let mut inner = imgui.inner.lock().unwrap();
    inner.id = 0;

    inner.render_list.clear();
}

fn add_widget<F, T>(f: F)
where
    F: FnOnce(u32) -> T,
    T: widgets::Widget + 'static,
{
    let imgui = imgui_inst();
    let mut inner = imgui.inner.lock().unwrap();
    inner.id += 1;
    let id: u32 = inner.id;

    if id as usize >= inner.render_list.len() {
        inner.render_list.push(Arc::new(f(id)));
    }
}

pub fn label(x: f32, y: f32, s: &str) {
    add_widget(|id| widgets::Label::new(id, x, y, s.into()));
}

#[derive(Default)]
pub struct Context {
    screen_size: (u32, u32),
    go: HashMap<u32, (Arc<widgets::Widget>, Rc<RefCell<GameObject>>)>,
}

impl Context {
    pub fn new(screen_w: u32, screen_h: u32) -> Context {
        Context {
            screen_size: (screen_w, screen_h),
            go: HashMap::new(),
        }
    }
}

pub fn pre_render(engine: &mut IEngine) {
    let imgui = imgui_inst();
    let inner = imgui.inner.lock().unwrap();
    let ctx = { &mut engine.gui_context() };

    let mut ctx_mut = ctx.borrow_mut();
    let (sw, sh) = ctx_mut.screen_size;
    let hm = &mut ctx_mut.go;

    for w in inner.render_list.iter() {
        match hm.get_mut(&w.id()) {
            None => {
                hm.insert(w.id(), (w.clone(), w.bind((sw, sh), engine)));
            }
            Some(&mut (ref oldw, _)) => {
                if **oldw != **w {
                    hm.insert(w.id(), (w.clone(), w.bind((sw, sh), engine)));
                }
            }
        };
    }
}

pub fn end() {}
