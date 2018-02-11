use na::Vector3;
use engine::core::ComponentBased;
use super::ShaderProgram;

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

    pub fn new<T>(a: T) -> Light
    where
        T: Into<Light>,
    {
        a.into()
    }

    pub fn bind(&self, lightname: &str, prog: &ShaderProgram) {
        match *self {
            Light::Directional(ref l) => l.bind(lightname, prog),
            Light::Point(ref l) => l.bind(lightname, prog),
        }
    }
}

impl ComponentBased for Light {}

pub struct DirectionalLight {
    pub direction: Vector3<f32>,
    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,
}

impl From<DirectionalLight> for Light {
    fn from(w: DirectionalLight) -> Light {
        Light::Directional(w)
    }
}

impl DirectionalLight {
    fn bind(&self, lightname: &str, prog: &ShaderProgram) {
        // We must have at least one direction light.
        prog.set(&format!("{}.direction", lightname), self.direction);
        prog.set(&format!("{}.ambient", lightname), self.ambient);
        prog.set(&format!("{}.diffuse", lightname), self.diffuse);
        prog.set(&format!("{}.specular", lightname), self.specular);
    }
}

pub struct PointLight {
    pub position: Vector3<f32>,

    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,

    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

impl From<PointLight> for Light {
    fn from(w: PointLight) -> Light {
        Light::Point(w)
    }
}

impl PointLight {
    fn bind(&self, lightname: &str, prog: &ShaderProgram) {
        // We must have at least one direction light.
        prog.set(&format!("{}.position", lightname), self.position);

        prog.set(&format!("{}.ambient", lightname), self.ambient);
        prog.set(&format!("{}.diffuse", lightname), self.diffuse);
        prog.set(&format!("{}.specular", lightname), self.specular);

        prog.set(&format!("{}.constant", lightname), self.constant);
        prog.set(&format!("{}.linear", lightname), self.linear);
        prog.set(&format!("{}.quadratic", lightname), self.quadratic);

        prog.set(&format!("{}.rate", lightname), 1.0);
    }
}
