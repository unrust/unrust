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
