use engine::asset::loader::{Loadable, Loader};
use engine::asset::{AssetError, AssetResult, AssetSystem, File};
use image::RgbaImage;
use image;

pub struct ImageLoader {}

impl Loadable for RgbaImage {
    type Loader = ImageLoader;
}

impl Loader<RgbaImage> for ImageLoader {
    fn load<A>(_asys: A, mut file: Box<File>) -> AssetResult<RgbaImage>
    where
        A: AssetSystem + Clone,
    {
        let buf = file.read_binary()
            .map_err(|_| AssetError::ReadBufferFail(file.name()))?;
        let img = image::load_from_memory(&buf).map_err(|e| AssetError::InvalidFormat {
            path: file.name(),
            len: buf.len(),
            reason: format!("{:?}", e),
        })?;
        Ok(img.to_rgba())
    }
}
