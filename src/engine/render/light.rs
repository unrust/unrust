use na::Vector3;
use engine::core::ComponentBased;
use super::ShaderProgram;
use na::Matrix4;
use alga::linear::Transformation;

pub enum Light {
    Directional(Directional),
    Point(Point),
}

macro_rules! impl_light {
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
    impl_light!(directional, directional_mut, Directional, Directional);
    impl_light!(point, point_mut, Point, Point);

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

pub struct Directional {
    pub direction: Vector3<f32>,
    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,
}

impl Default for Directional {
    fn default() -> Directional {
        let m = Matrix4::from_euler_angles(30.0_f32.to_radians(), 50.0_f32.to_radians(), 0.0);
        let light_dir = Vector3::new(0.0, 0.0, 1.0);
        let light_dir = m.transform_vector(&light_dir);

        Directional {
            direction: light_dir.normalize(),
            ambient: Vector3::new(0.212, 0.227, 0.259),
            diffuse: Vector3::new(1.0, 0.957, 0.839),
            specular: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl From<Directional> for Light {
    fn from(w: Directional) -> Light {
        Light::Directional(w)
    }
}

impl Directional {
    fn bind(&self, lightname: &str, prog: &ShaderProgram) {
        prog.set(&(lightname.to_string() + ".direction"), self.direction);
        prog.set(&(lightname.to_string() + ".ambient"), self.ambient);
        prog.set(&(lightname.to_string() + ".diffuse"), self.diffuse);
        prog.set(&(lightname.to_string() + ".specular"), self.specular);
    }
}

pub struct Point {
    pub position: Vector3<f32>,

    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,

    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

impl From<Point> for Light {
    fn from(w: Point) -> Light {
        Light::Point(w)
    }
}

impl Default for Point {
    fn default() -> Point {
        Point {
            position: Vector3::new(0.0, 0.0, 0.0),
            ambient: Vector3::new(0.05, 0.05, 0.05),
            diffuse: Vector3::new(0.8, 0.8, 0.8),
            specular: Vector3::new(1.0, 1.0, 1.0),
            constant: 1.0,
            linear: 0.022,
            quadratic: 0.0019,
        }
    }
}

impl Point {
    fn bind(&self, lightname: &str, prog: &ShaderProgram) {
        prog.set(&(lightname.to_string() + ".position"), self.position);

        prog.set(&(lightname.to_string() + ".ambient"), self.ambient);
        prog.set(&(lightname.to_string() + ".diffuse"), self.diffuse);
        prog.set(&(lightname.to_string() + ".specular"), self.specular);

        prog.set(&(lightname.to_string() + ".constant"), self.constant);
        prog.set(&(lightname.to_string() + ".linear"), self.linear);
        prog.set(&(lightname.to_string() + ".quadratic"), self.quadratic);

        prog.set(&(lightname.to_string() + ".rate"), 1.0);
    }
}
