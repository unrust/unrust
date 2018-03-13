use std::default::Default;
use futures::prelude::*;
use std;

pub type FileFuture = Box<Future<Item = Box<File>, Error = FileIoError>>;

pub trait FileSystem: Default {
    type File;

    fn open(&self, filename: &str) -> FileFuture;

    fn loading_files(&self) -> Vec<String>;
}

pub trait File {
    fn name(&self) -> String;

    fn read_binary(&mut self) -> Result<Vec<u8>, FileIoError>;
}

#[derive(Debug)]
pub enum FileIoError {
    NotReady,
    NoSuchFile(String),
    IoError(std::io::Error),
    Unknown(String),
}

impl From<std::io::Error> for FileIoError {
    fn from(e: std::io::Error) -> FileIoError {
        FileIoError::IoError(e)
    }
}
