use std::default::Default;
use futures::prelude::*;

pub type FileFuture = Box<Future<Item = Box<File>, Error = FileIoError>>;

pub trait FileSystem: Default {
    type File;

    fn open(&self, filename: &str) -> FileFuture;
}

pub trait File {
    fn name(&self) -> String;

    fn read_binary(&mut self) -> Result<Vec<u8>, FileIoError>;
}

#[derive(Debug, Clone)]
pub enum FileIoError {
    NotReady,
    NoSuchFile,
    Unknown,
}
