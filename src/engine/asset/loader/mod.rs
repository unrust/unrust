mod loader;
mod image;
mod shader;

pub use self::loader::{Loadable, Loader};
pub use self::image::ImageLoader;
pub use self::shader::{ShaderFSLoader, ShaderVSLoader};
