use math::*;
use engine::MeshData;

pub trait QuadBuilder {
    fn add_quad(&mut self, ps: [Vector3f; 4]);
}

fn add_v(v: &mut Vec<f32>, p: &Vector3f) {
    v.push(p.x);
    v.push(p.y);
    v.push(p.z);
}

impl QuadBuilder for MeshData {
    fn add_quad(&mut self, ps: [Vector3f; 4]) {
        add_v(&mut self.vertices, &ps[0]);
        add_v(&mut self.vertices, &ps[1]);
        add_v(&mut self.vertices, &ps[2]);

        add_v(&mut self.vertices, &ps[2]);
        add_v(&mut self.vertices, &ps[3]);
        add_v(&mut self.vertices, &ps[0]);

        self.indices.push(self.indices.len() as u16);
        self.indices.push(self.indices.len() as u16);
        self.indices.push(self.indices.len() as u16);

        self.indices.push(self.indices.len() as u16);
        self.indices.push(self.indices.len() as u16);
        self.indices.push(self.indices.len() as u16);
    }
}
