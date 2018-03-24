use math::Vector3f;
use std::default::Default;

#[derive(Copy, Clone, Debug)]
pub struct Aabb {
    pub min: Vector3f,
    pub max: Vector3f,
}

impl Default for Aabb {
    fn default() -> Aabb {
        Aabb::empty()
    }
}

impl Aabb {
    pub fn empty() -> Self {
        use std::f32::{MAX, MIN};

        Self {
            min: Vector3f::new(MAX, MAX, MAX),
            max: Vector3f::new(MIN, MIN, MIN),
        }
    }

    pub fn merge(&mut self, other: &Self) {
        self.min[0] = self.min[0].min(other.min[0]);
        self.min[1] = self.min[1].min(other.min[1]);
        self.min[2] = self.min[2].min(other.min[2]);

        self.max[0] = self.max[0].max(other.max[0]);
        self.max[1] = self.max[1].max(other.max[1]);
        self.max[2] = self.max[2].max(other.max[2]);
    }

    pub fn merge_point(&mut self, p: &Vector3f) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.min.z = self.min.z.min(p.z);

        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
        self.max.z = self.max.z.max(p.z);
    }

    pub fn merge_sphere(&mut self, p: &Vector3f, r: f32) {
        self.merge(&Aabb {
            min: p + Vector3f::new(-r, -r, -r),
            max: p + Vector3f::new(r, r, r),
        });
    }

    pub fn corners(&self) -> [Vector3f; 8] {
        [
            Vector3f::new(self.min.x, self.min.y, self.min.z),
            Vector3f::new(self.max.x, self.min.y, self.min.z),
            Vector3f::new(self.max.x, self.max.y, self.min.z),
            Vector3f::new(self.min.x, self.max.y, self.min.z),
            Vector3f::new(self.min.x, self.min.y, self.max.z),
            Vector3f::new(self.max.x, self.min.y, self.max.z),
            Vector3f::new(self.max.x, self.max.y, self.max.z),
            Vector3f::new(self.min.x, self.max.y, self.max.z),
        ]
    }
}
