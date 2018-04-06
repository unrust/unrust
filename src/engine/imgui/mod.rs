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

mod context;
mod image;
mod instance;
mod label;
mod metric;
mod widgets;

use engine::IEngine;
use engine::render::{Material, Texture};
use std::rc::Rc;

pub use self::context::Context;
pub use self::metric::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TextAlign {
    Left,
    Right,
    Center,
}

impl Default for TextAlign {
    fn default() -> TextAlign {
        TextAlign::Left
    }
}

pub fn begin() {
    let imgui = instance::imgui_inst();
    let mut inner = imgui.inner.lock().unwrap();
    inner.id = 0;

    inner.render_list.clear();
}

fn add_widget<F>(f: F)
where
    F: FnOnce(u32, instance::ImguiState) -> widgets::Widget,
{
    let imgui = instance::imgui_inst();
    let mut inner = imgui.inner.lock().unwrap();
    inner.id += 1;

    let id: u32 = inner.id;
    let state = inner.state;

    if id as usize >= inner.render_list.len() {
        inner.render_list.push(Rc::new(f(id, state)));
    }
}

/// Pivot controls how to place the ui element
pub fn pivot(p: (f32, f32)) {
    let imgui = instance::imgui_inst();
    let mut inner = imgui.inner.lock().unwrap();
    inner.state.pivot = Metric::Native(p.0, p.1);
}

/// Text align setting
pub fn text_align(align: TextAlign) {
    let imgui = instance::imgui_inst();
    let mut inner = imgui.inner.lock().unwrap();
    inner.state.text_align = align;
}

/// Label
pub fn label(pos: Metric, s: &str) {
    add_widget(|id, state| label::Label::new(id, pos, state, s.into()));

    // reset text settings
    text_align(TextAlign::default());
}

/// Image
pub fn image(pos: Metric, size: Metric, tex: Rc<Texture>) {
    add_widget(|id, state| image::Image::new(id, pos, size, state, tex));
}

/// Image with material
pub fn image_with_material(pos: Metric, size: Metric, material: Rc<Material>) {
    add_widget(|id, state| image::Image::new(id, pos, size, state, material));
}

pub fn pre_render(engine: &mut IEngine) {
    let imgui = instance::imgui_inst();
    let mut inner = imgui.inner.lock().unwrap();

    let ctx = { &mut engine.gui_context() };
    let mut ctx_mut = ctx.borrow_mut();

    ctx_mut.update(&mut inner, engine);
}

pub fn end() {}
