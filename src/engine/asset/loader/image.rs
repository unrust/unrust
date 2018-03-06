use engine::asset::loader::{Loadable, Loader};
use engine::asset::{AssetError, AssetResult, AssetSystem, File};
use image::RgbaImage;
use image;
use uni_app;

pub struct ImageLoader {}

impl Loadable for RgbaImage {
    type Loader = ImageLoader;
}

impl Loader<RgbaImage> for ImageLoader {
    fn load<A>(_asys: A, mut file: Box<File>) -> AssetResult<RgbaImage>
    where
        A: AssetSystem + Clone,
    {
        let t = uni_app::now();

        let buf = file.read_binary()
            .map_err(|_| AssetError::ReadBufferFail(file.name()))?;
        let img = image::load_from_memory(&buf).map_err(|e| AssetError::InvalidFormat {
            path: file.name(),
            len: buf.len(),
            reason: format!("{:?}", e),
        })?;
        let rgba = img.to_rgba();

        uni_app::App::print(format!(
            "image {} loading time : {}\n",
            file.name(),
            (uni_app::now() - t)
        ));

        Ok(rgba)
    }
}
