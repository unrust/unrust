//! imgui crate
//!
//! `imgui` is a collection of utilites to make simple UI element
//! The top-left of screen is (0.0,0.0) and the bottom-right is (1.0,1.0)
//!
//! Supported elements
//!
//! Label
//!
//! Positioning
//!     Pivot to control how the element is positiion related to itself.
//!     E.g: let the `position` of the element is (x,y)
//!     pivot(0,0) => represent the top-left corner of element will be placed in (x,y)
//!     pivot(1,1) => represent the bottom-right corner of element will be place in (x,y)
//!
//!

mod widgets;

use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

use engine::core::GameObject;
use engine::IEngine;
use engine::render::Texture;
use std::collections::HashMap;
use std::ops::{Add, Sub};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Metric {
    /// Left Top =(0,0), Right, Bottom = (1,1)
    Native(f32, f32),
    /// Left Top = (0,0), Right, Bottom = (screen width,screen height)
    Pixel(f32, f32),
    Mixed((f32, f32), (f32, f32)),
}

impl Default for Metric {
    fn default() -> Metric {
        Metric::Native(0.0, 0.0)
    }
}

impl From<(f32, f32)> for Metric {
    fn from(p: (f32, f32)) -> Self {
        Metric::Native(p.0, p.1)
    }
}

impl Add for Metric {
    type Output = Metric;

    fn add(self, other: Metric) -> Metric {
        match self {
            Metric::Native(x, y) => match other {
                Metric::Native(ox, oy) => Metric::Native(x + ox, y + oy),
                Metric::Pixel(ox, oy) => Metric::Mixed((x, y), (ox, oy)),
                Metric::Mixed((oax, oay), b) => Metric::Mixed((oax + x, oay + y), b),
            },

            Metric::Pixel(x, y) => match other {
                Metric::Native(ox, oy) => Metric::Mixed((ox, oy), (x, y)),
                Metric::Pixel(ox, oy) => Metric::Pixel(x + ox, y + oy),
                Metric::Mixed(a, (obx, oby)) => Metric::Mixed(a, (obx + x, oby + x)),
            },

            Metric::Mixed((ax, ay), (bx, by)) => match other {
                Metric::Native(ox, oy) => Metric::Mixed((ax + ox, ay + oy), (bx, by)),
                Metric::Pixel(ox, oy) => Metric::Mixed((ax, ay), (bx + ox, by + oy)),
                Metric::Mixed((oax, oay), (obx, oby)) => {
                    Metric::Mixed((ax + oax, ay + oay), (bx + obx, by + oby))
                }
            },
        }
    }
}

impl Sub for Metric {
    type Output = Metric;

    fn sub(self, other: Metric) -> Metric {
        match other {
            Metric::Native(px, py) => self + Metric::Native(-px, -py),
            Metric::Pixel(px, py) => self + Metric::Pixel(-px, -py),
            Metric::Mixed((ax, ay), (bx, by)) => self + Metric::Mixed((-ax, -ay), (-bx, -by)),
        }
    }
}

#[derive(Default, Debug)]
struct ImguiRaw {
    id: u32,
    pivot: Metric,
    render_list: Vec<Arc<widgets::Widget>>,
}

struct Imgui {
    inner: Arc<Mutex<ImguiRaw>>,
}

thread_local!(
    static INSTANCE: Arc<Mutex<ImguiRaw>> = Arc::new(Mutex::new(Default::default()))
);

fn imgui_inst() -> Imgui {
    return Imgui {
        inner: INSTANCE.with(|f| f.clone()),
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
    F: FnOnce(u32, Metric) -> T,
    T: widgets::Widget + 'static,
{
    let imgui = imgui_inst();
    let mut inner = imgui.inner.lock().unwrap();
    inner.id += 1;

    let id: u32 = inner.id;
    let pivot: Metric = inner.pivot;

    if id as usize >= inner.render_list.len() {
        inner.render_list.push(Arc::new(f(id, pivot)));
    }
}

/// Pivot controls how to place the ui element
/// It si

pub fn pivot(p: (f32, f32)) {
    let imgui = imgui_inst();
    let mut inner = imgui.inner.lock().unwrap();
    inner.pivot = Metric::Native(p.0, p.1);
}

/// Label
pub fn label(pos: Metric, s: &str) {
    add_widget(|id, pivot| widgets::Label::new(id, pos, pivot, s.into()));
}

/// Image
pub fn image(pos: Metric, size: Metric, tex: Rc<Texture>) {
    add_widget(|id, pivot| widgets::Image::new(id, pos, size, pivot, tex));
}

type WidgetGoMap = HashMap<u32, (Arc<widgets::Widget>, Rc<RefCell<GameObject>>)>;

#[derive(Default)]
pub struct Context {
    screen_size: (u32, u32),
    go: WidgetGoMap,
}

impl Context {
    pub fn new(screen_w: u32, screen_h: u32) -> Context {
        Context {
            screen_size: (screen_w, screen_h),
            go: HashMap::new(),
        }
    }
}

fn strip_cache(curr: u32, hm: &mut WidgetGoMap) {
    let empties: Vec<_> = hm.iter()
        .filter(|&(k, _)| *k > curr)
        .map(|(k, _)| k.clone())
        .collect();

    for empty in empties {
        hm.remove(&empty);
    }
}

pub fn pre_render(engine: &mut IEngine) {
    let imgui = imgui_inst();
    let inner = imgui.inner.lock().unwrap();
    let ctx = { &mut engine.gui_context() };

    let mut ctx_mut = ctx.borrow_mut();
    let (sw, sh) = ctx_mut.screen_size;

    {
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

    // remove all go in hm which id >= last id
    strip_cache(inner.id, &mut ctx_mut.go);
}

pub fn end() {}
