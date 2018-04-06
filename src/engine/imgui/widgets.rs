use engine::core::Aabb;
use math::*;

use super::Metric;
use std::fmt::Debug;

use super::image;
use super::label;

pub trait WidgetBinder: Debug {
    fn id(&self) -> u32;
    fn is_same(&self, other: &Widget) -> bool;
}

#[derive(Debug)]
pub enum Widget {
    Image(image::Image),
    Label(label::Label),
}

impl Widget {
    pub fn id(&self) -> u32 {
        match self {
            &Widget::Image(ref img) => img.id(),
            &Widget::Label(ref lbl) => lbl.id(),
        }
    }
}

impl PartialEq for Widget {
    fn eq(&self, other: &Widget) -> bool {
        match self {
            &Widget::Image(ref img) => img.is_same(other),
            &Widget::Label(ref lbl) => lbl.is_same(other),
        }
    }
}

pub fn to_pixel_pos(px: f32, py: f32, ssize: &(u32, u32), hidpi: f32) -> (f32, f32) {
    ((
        (px * 2.0 * hidpi) / (ssize.0 as f32),
        (py * 2.0 * hidpi) / (ssize.1 as f32),
    ))
}

pub fn compute_translate(
    pos: &Metric,
    pivot: &Metric,
    ssize: &(u32, u32),
    hidpi: f32,
    bounds: &Aabb,
) -> Vector3<f32> {
    let w = bounds.max.x - bounds.min.x;
    let h = bounds.max.y - bounds.min.y;

    let (x, y) = match pos {
        &Metric::Native(px, py) => (px * 2.0, py * 2.0),
        &Metric::Pixel(px, py) => to_pixel_pos(px, py, ssize, hidpi),
        &Metric::Mixed((ax, ay), (bx, by)) => {
            let vp = to_pixel_pos(bx, by, ssize, hidpi);
            (ax * 2.0 + vp.0, ay * 2.0 + vp.1)
        }
    };

    let (offsetx, offsety) = match pivot {
        &Metric::Native(px, py) => (px * w, py * h),
        _ => unreachable!(),
    };

    Vector3::new(x - 1.0 - offsetx, y * -1.0 + 1.0 + offsety, 0.0)
}
