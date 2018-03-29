use std::rc::Rc;
use engine::render::{RenderQueue, RenderTexture};
use engine::core::ComponentBased;
use std::collections::BTreeSet;
use math::*;

pub struct Plane {
    n: Vector3<f32>,
    offset: f32,
}

impl Plane {
    /// Make a plane from 3 points (anti-clockwise)
    pub fn from_3_points(p0: &Vector3<f32>, p1: &Vector3<f32>, p2: &Vector3<f32>) -> Plane {
        let n = (p1 - p0).cross(p2 - p1).normalize();
        let offset = n.dot(*p0);

        Plane { n, offset }
    }
}

pub struct Frustum {
    planes: [Plane; 6],
}

impl Frustum {
    pub fn collide_sphere(&self, p: &Vector3<f32>, r: f32) -> bool {
        for plane in self.planes.iter() {
            // Distance = (A*x0+B*y0+C*z0+D)/Sqrt(A*A+B*B+C*C)
            if plane.n.dot(*p) - plane.offset < -r {
                return false;
            }
        }

        true
    }
}

pub struct Camera {
    pub v: Matrix4<f32>,

    pub enable_frustum_culling: bool,

    /// Optional viewport of this camera,  (pos, size) in pixels
    /// from 0 (left/top) to screen width/height (right/bottom)
    pub rect: Option<((i32, i32), (u32, u32))>,
    pub znear: f32,
    pub zfar: f32,

    pub included_render_queues: Option<BTreeSet<RenderQueue>>,

    eye: Point3<f32>,

    pub render_texture: Option<Rc<RenderTexture>>,
}

impl Default for Camera {
    fn default() -> Camera {
        let cam = Camera::new();
        cam
    }
}

fn extract_forward(m: &Matrix4<f32>) -> Vector3<f32> {
    //-Vector3::new(m.data[2], m.data[2 + 1 * 4], m.data[2 + 2 * 4])
    -m.row(2).truncate()
}

fn extract_up(m: &Matrix4<f32>) -> Vector3<f32> {
    //Vector3::new(m.data[1], m.data[1 + 1 * 4], m.data[1 + 2 * 4])
    m.row(1).truncate()
}

fn extract_right(m: &Matrix4<f32>) -> Vector3<f32> {
    //Vector3::new(m.data[0], m.data[0 + 1 * 4], m.data[0 + 2 * 4])
    m.row(0).truncate()
}

impl ComponentBased for Camera {}

impl Camera {
    pub fn forward(&self) -> Vector3<f32> {
        extract_forward(&self.v)
    }

    pub fn lookat(&mut self, eye: &Point3<f32>, target: &Point3<f32>, up: &Vector3<f32>) {
        self.v = Matrix4::look_at(*eye, *target, *up);
        self.eye = *eye;

        // let g_forward = extract_forward(&self.v);
        // let forward = (target - eye).normalize();
        // debug_assert!(
        //     (forward.dot(&g_forward) - 1.0).abs() < 0.001,
        //     format!("forwards: {:?}, {:?} {:?}", forward, g_forward, self.v)
        // );

        // let g_right = extract_right(&self.v);
        // let right = forward.cross(&up).normalize();
        // debug_assert!(
        //     (right.dot(&g_right) - 1.0).abs() < 0.001,
        //     format!("rights: {:?}, {:?} {:?}", right, g_right, self.v)
        // );

        // let g_up = extract_up(&self.v);
        // let up = right.cross(&forward).normalize();
        // debug_assert!(
        //     (up.dot(&g_up) - 1.0).abs() < 0.001,
        //     format!("ups: {:?}, {:?} {:?}", up, g_up, self.v)
        // );
    }

    fn calc_aspect(&self, screen_size: (u32, u32)) -> f32 {
        let mut aspect: f32 = (screen_size.0 as f32) / (screen_size.1 as f32);

        if let Some(((_, _), (w, h))) = self.rect {
            aspect = w as f32 / h as f32;
        }

        aspect
    }

    pub fn perspective(&self, screen_size: (u32, u32)) -> Matrix4<f32> {
        use math::*;

        let aspect = self.calc_aspect(screen_size);

        PerspectiveFov {
            fovy: Rad(3.1415 / 4.0),
            aspect,
            near: self.znear,
            far: self.zfar,
        }.into()
    }

    pub fn new() -> Camera {
        Camera {
            v: Matrix4::identity(),
            eye: Point3::new(0.0, 0.0, 0.0),
            rect: None,
            znear: 0.03,
            zfar: 1000.0,
            enable_frustum_culling: true,
            included_render_queues: None,
            render_texture: None,
        }
    }

    pub fn eye(&self) -> Vector3<f32> {
        Vector3::new(self.eye.x, self.eye.y, self.eye.z)
    }

    pub fn calc_frustum(&self, screen_size: (u32, u32)) -> Frustum {
        let forward = extract_forward(&self.v);
        let up = extract_up(&self.v);
        let right = extract_right(&self.v);
        let aspect = self.calc_aspect(screen_size);

        let near_center = self.eye.to_vec() + forward * self.znear;
        let far_center = self.eye.to_vec() + forward * self.zfar;

        let fovy: f32 = 3.1415 / 4.0;

        let near_height = 2.0 * (fovy * 0.5).tan() * self.znear;
        let far_height = 2.0 * (fovy * 0.5).tan() * self.zfar;
        let near_width = near_height * aspect;
        let far_width = far_height * aspect;

        let far_top_left = far_center + up * (far_height * 0.5) - right * (far_width * 0.5);
        let far_top_right = far_center + up * (far_height * 0.5) + right * (far_width * 0.5);
        let far_bottom_left = far_center - up * (far_height * 0.5) - right * (far_width * 0.5);
        let far_bottom_right = far_center - up * (far_height * 0.5) + right * (far_width * 0.5);

        let near_top_left = near_center + up * (near_height * 0.5) - right * (near_width * 0.5);
        let near_top_right = near_center + up * (near_height * 0.5) + right * (near_width * 0.5);
        let near_bottom_left = near_center - up * (near_height * 0.5) - right * (near_width * 0.5);
        let near_bottom_right = near_center - up * (near_height * 0.5) + right * (near_width * 0.5);

        let plane_left = Plane::from_3_points(&near_bottom_left, &far_bottom_left, &far_top_left);
        let plane_right = Plane::from_3_points(&near_bottom_right, &near_top_right, &far_top_right);
        let plane_top = Plane::from_3_points(&near_top_right, &near_top_left, &far_top_left);
        let plane_bottom =
            Plane::from_3_points(&near_bottom_left, &near_bottom_right, &far_bottom_right);
        let plane_near = Plane::from_3_points(&near_bottom_left, &near_top_left, &near_top_right);
        let plane_far = Plane::from_3_points(&far_bottom_left, &far_bottom_right, &far_top_right);

        Frustum {
            planes: [
                plane_left,
                plane_right,
                plane_top,
                plane_bottom,
                plane_near,
                plane_far,
            ],
        }
    }
}
