use std::default::Default;

pub trait FileSystem: Default {
    type File;

    fn open(&self, filename: &str) -> Result<Self::File, FileIoError>;
}

pub trait File {
    fn name(&self) -> String;

    fn is_ready(&self) -> bool;

    fn read_binary(&mut self) -> Result<Vec<u8>, FileIoError>;
}

#[derive(Debug)]
pub enum FileIoError {
    NotReady,
    NoSuchFile,
    InvalidFormat,
}
