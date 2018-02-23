use engine::render::MeshData;

pub struct Quad {}

impl Quad {
    pub fn new() -> MeshData {
        let vertices: Vec<f32> = vec![
            -1.0, 1.0, 0.0,     // 0
            -1.0, -1.0, 0.0,    // 1
            1.0, -1.0, 0.0,     // 2
            1.0, 1.0, 0.0       // 3
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

        MeshData {
            vertices: vertices,
            uvs: Some(uvs),
            normals: None,
            indices: indices,
        }
    }
}
