use engine::asset::loader::{Loadable, Loader};
use engine::asset::{AssetError, File};
use image::RgbaImage;
use image;

pub struct ImageLoader {}

impl Loadable for RgbaImage {
    type Loader = ImageLoader;
}

impl Loader<RgbaImage> for ImageLoader {
    fn load(mut file: Box<File>) -> Result<RgbaImage, AssetError> {
        let buf = file.read_binary()
            .map_err(|_| AssetError::InvalidFormat(file.name()))?;
        let img =
            image::load_from_memory(&buf).map_err(|_| AssetError::InvalidFormat(file.name()))?;
        Ok(img.to_rgba())
    }
}
