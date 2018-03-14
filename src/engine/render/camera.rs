use na::{Matrix4, Point3, Vector3};
use std::rc::Rc;
use engine::render::RenderTexture;
use engine::core::ComponentBased;

pub struct Camera {
    pub v: Matrix4<f32>,

    /// Optional viewport of this camera,  (pos, size) in pixels
    /// from 0 (left/top) to screen width/height (right/bottom)
    pub rect: Option<((i32, i32), (u32, u32))>,
    pub znear: f32,
    pub zfar: f32,
    eye: Point3<f32>,
    pub render_texture: Option<Rc<RenderTexture>>,
}

impl Default for Camera {
    fn default() -> Camera {
        let cam = Camera::new();
        cam
    }
}

impl ComponentBased for Camera {}

impl Camera {
    pub fn lookat(&mut self, eye: &Point3<f32>, target: &Point3<f32>, up: &Vector3<f32>) {
        self.v = Matrix4::look_at_rh(eye, target, up);
        self.eye = *eye;
    }

    pub fn perspective(&self, screen_size: (u32, u32)) -> Matrix4<f32> {
        let mut aspect: f32 = (screen_size.0 as f32) / (screen_size.1 as f32);

        if let Some(((_, _), (w, h))) = self.rect {
            aspect = w as f32 / h as f32;
        }

        Matrix4::new_perspective(aspect, 3.1415 / 4.0, self.znear, self.zfar)
    }

    pub fn new() -> Camera {
        Camera {
            v: Matrix4::identity(),
            eye: Point3::new(0.0, 0.0, 0.0),
            rect: None,
            znear: 0.03,
            zfar: 1000.0,
            render_texture: None,
        }
    }

    pub fn eye(&self) -> Vector3<f32> {
        Vector3::new(self.eye.x, self.eye.y, self.eye.z)
    }
}
