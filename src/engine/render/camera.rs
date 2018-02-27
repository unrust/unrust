use na::{Matrix4, Point3, Vector3};
use std::rc::Rc;
use engine::render::RenderTexture;

pub struct Camera {
    pub v: Matrix4<f32>,
    pub p: Matrix4<f32>,

    /// Optional viewport of this camera,  (pos, size) in pixels
    /// from 0 (left/top) to screen width/height (right/bottom)
    pub rect: Option<((i32, i32), (u32, u32))>,

    eye: Point3<f32>,
    pub render_texture: Option<Rc<RenderTexture>>,
}

impl Camera {
    pub fn lookat(&mut self, eye: &Point3<f32>, target: &Point3<f32>, up: &Vector3<f32>) {
        self.v = Matrix4::look_at_rh(eye, target, up);
        let mut aspect: f32 = 800.0/600.0;
        if let Some(((_,_),(w,h))) = self.rect {
            aspect = w as f32 / h as f32;
        }
        self.p = Matrix4::new_perspective(aspect, 3.1415 / 4.0, 1.0, 1000.0);
        self.eye = *eye;
    }

    pub fn new() -> Camera {
        Camera {
            v: Matrix4::identity(),
            p: Matrix4::identity(),
            eye: Point3::new(0.0, 0.0, 0.0),
            rect: None,
            render_texture: None,
        }
    }

    pub fn eye(&self) -> Vector3<f32> {
        Vector3::new(self.eye.x, self.eye.y, self.eye.z)
    }
}
