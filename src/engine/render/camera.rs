use na::{Matrix4, Point3, Vector3};

pub struct Camera {
    pub v: Matrix4<f32>,
    pub p: Matrix4<f32>,
}

impl Camera {
    pub fn lookat(&mut self, eye: &Point3<f32>, target: &Point3<f32>, up: &Vector3<f32>) {
        self.v = Matrix4::look_at_rh(eye, target, up);
        self.p = Matrix4::new_perspective(800.0 / 600.0, 3.1415 / 4.0, 1.0, 1000.0);
    }

    pub fn new() -> Camera {
        Camera {
            v: Matrix4::identity(),
            p: Matrix4::identity(),
        }
    }
}
