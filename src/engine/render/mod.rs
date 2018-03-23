mod camera;
mod mesh;
mod shader_program;
mod texture;
mod material;
mod light;
mod shader;
mod uniforms;
mod frame_buffer;
mod render_texture;
mod mesh_buffer;

#[derive(Hash, Eq, Ord, PartialOrd, PartialEq, Copy, Clone, Debug)]
pub enum RenderQueue {
    Opaque = 1000,
    Skybox = 2000,
    Transparent = 3000,
    UI = 5000,
}

pub mod mesh_util;

pub use self::camera::{Camera, Frustum};
pub use self::shader::{ShaderFs, ShaderKind, ShaderVs};
pub use self::shader_program::ShaderProgram;
pub use self::texture::{Texture, TextureAsset, TextureAttachment, TextureFiltering, TextureImage,
                        TextureWrap};
pub use self::mesh::{Mesh, MeshSurface};
pub use self::mesh_buffer::{MeshBuffer, MeshData};
pub use self::material::{CullMode, DepthTest, Material, MaterialParam, MaterialState};
pub use self::light::{Directional, Light, Point};
pub use self::render_texture::RenderTexture;
