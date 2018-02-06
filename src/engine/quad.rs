use Mesh;
use MeshBuffer;
use Component;
use std::sync::Arc;

pub struct Quad(MeshBuffer);

impl Quad {
    pub fn new_quad_component() -> Arc<Component> {
        Component::new(Quad::new_quad())
    }

    pub fn new_quad() -> Mesh {
        let vertices: Vec<f32> = vec![
            -1.0, 1.0, 0.0, -1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0
        ];

        let uvs: Vec<f32> = vec![
            // Top face
            0.0, 0.0,
            0.0, 1.0,
            1.0, 1.0,
            1.0, 0.0,
        ];

        let indices: Vec<u16> = vec![
            0, 1, 2, 0, 2, 3 // Top face
        ];

        Mesh::new(MeshBuffer {
            vertices: vertices,
            uvs: Some(uvs),
            normals: None,
            indices: indices,
        })
    }
}
