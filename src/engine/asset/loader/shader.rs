use engine::asset::loader::Loader;
use engine::asset::{AssetError, AssetResult, File};
use engine::render::{ShaderFs, ShaderVs};

use std::str;

pub struct ShaderVSLoader {}
pub struct ShaderFSLoader {}

impl Loader<ShaderVs> for ShaderVSLoader {
    fn load<A>(_asys: A, mut file: Box<File>) -> AssetResult<ShaderVs> {
        let buf = file.read_binary()
            .map_err(|_| AssetError::ReadBufferFail(file.name()))?;
        let vs = str::from_utf8(&buf).map_err(|e| AssetError::InvalidFormat {
            path: file.name(),
            len: buf.len(),
            reason: format!("{:?}", e),
        })?;
        Ok(ShaderVs::new(&file.name(), vs))
    }
}

impl Loader<ShaderFs> for ShaderFSLoader {
    fn load<A>(_asys: A, mut file: Box<File>) -> AssetResult<ShaderFs> {
        let buf = file.read_binary()
            .map_err(|_| AssetError::ReadBufferFail(file.name()))?;
        let fs = str::from_utf8(&buf).map_err(|e| AssetError::InvalidFormat {
            path: file.name(),
            len: buf.len(),
            reason: format!("{:?}", e),
        })?;

        Ok(ShaderFs::new(&file.name(), fs))
    }
}
