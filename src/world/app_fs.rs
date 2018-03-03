use engine::{Engine, File, FileFuture, FileIoError, FileSystem};
use uni_app::fs;

use futures::{Async, Future};
use futures::future;

// unrust engine support different file system.
#[derive(Default)]
pub struct AppFileSystem {}
pub struct AppFile(String, fs::File);
pub struct AppFileReader(Option<AppFile>);

impl Future for AppFileReader {
    type Item = Box<File>;
    type Error = FileIoError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if !self.0.as_ref().unwrap().is_ready() {
            return Ok(Async::NotReady);
        }

        if let Some(f) = self.0.take() {
            return Ok(Async::Ready(Box::new(f)));
        }

        unreachable!();
    }
}

impl FileSystem for AppFileSystem {
    type File = AppFile;

    fn open(&self, filename: &str) -> FileFuture {
        let f = fs::FileSystem::open(filename)
            .map_err(|_| FileIoError::NoSuchFile(filename.to_string()));

        match f {
            Err(e) => Box::new(future::err(e)),
            Ok(file) => Box::new(AppFileReader(Some(AppFile(filename.into(), file)))),
        }
    }
}

impl File for AppFile {
    fn name(&self) -> String {
        self.0.clone()
    }

    fn read_binary(&mut self) -> Result<Vec<u8>, FileIoError> {
        self.1.read_binary().map_err(|_| FileIoError::NotReady)
    }
}

impl AppFile {
    fn is_ready(&self) -> bool {
        self.1.is_ready()
    }
}

pub type AppEngine = Engine<AppFileSystem, AppFile>;
