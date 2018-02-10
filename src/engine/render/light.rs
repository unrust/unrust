use na::Vector3;
use engine::core::ComponentBased;

pub enum Light {
    Directional(DirectionalLight),
    Point(PointLight),
}

macro_rules! add_light_cast {
    ($s:ident, $sm:ident, $v:ident, $t:ty) => {
        pub fn $s(&self) -> Option<&$t> {
            if let &Light::$v(ref l) = self {
                Some(l)
            } else {
                None
            }
        }

        pub fn $sm(&mut self) -> Option<&mut $t> {
            if let &mut Light::$v(ref mut l) = self {
                Some(l)
            } else {
                None
            }
        }
    };
}

impl Light {
    add_light_cast!(directional, directional_mut, Directional, DirectionalLight);
    add_light_cast!(point, point_mut, Point, PointLight);
}

pub struct DirectionalLight {
    pub dir: Vector3<f32>,
}

pub struct PointLight {
    pub pos: Vector3<f32>,
}

impl ComponentBased for Light {}
