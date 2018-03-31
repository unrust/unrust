use std;
use std::io::Read;
use std::io::ErrorKind;
use std::str;

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
    pub fn read_text(&mut self) -> Result<String, IoError> {
        let mut data = String::new();
        match self.0.read_to_string(&mut data) {
            Ok(_) => Ok(data),
            Err(e) => Err(std::io::Error::new(ErrorKind::Other, e)),
        }
    }
    pub fn is_ready(&self) -> bool {
        true
    }
}
