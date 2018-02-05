use std;
use std::io::Read;

pub struct FileSystem {}
pub struct File(std::fs::File);
pub type IoError = std::io::Error;
pub type IoErrorKind = std::io::ErrorKind;

impl FileSystem {
    pub fn open(s: &str) -> Result<File, IoError> {
        let file = std::fs::File::open(s)?;
        Ok(File(file))
    }
}

impl File {
    pub fn read_binary(&mut self) -> Result<Vec<u8>, IoError> {
        let mut buf = Vec::new();
        self.0.read_to_end(&mut buf)?;
        Ok(buf)
    }

    pub fn is_ready(&self) -> bool {
        true
    }
}
