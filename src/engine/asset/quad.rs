use engine::render::MeshBuffer;

pub struct Quad {}

impl Quad {
    pub fn new() -> MeshBuffer {
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

        let mut m = MeshBuffer::default();
        m.vertices = vertices;
        m.uvs = Some(uvs);
        m.normals = None;
        m.indices = indices;
        m
    }
}
