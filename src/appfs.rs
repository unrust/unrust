use uni_app;
use unigame::engine::{Engine, File, FileFuture, FileIoError, FileSystem};

use futures::{Async, Future};
use std::mem;

// unigame engine support different file system.
#[derive(Default)]
pub struct AppFileSystem {}
pub struct AppFile(String, uni_app::fs::File);
pub struct AppFileReader(Result<AppFile, FileIoError>);

impl Future for AppFileReader {
    type Item = Box<File>;
    type Error = FileIoError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        {
            let f = self.0.as_ref().map_err(|err| err.clone())?;

            if !f.is_ready() {
                return Ok(Async::NotReady);
            }
        }

        let r = mem::replace(&mut self.0, Err(FileIoError::NoSuchFile));

        if let Ok(f) = r {
            return Ok(Async::Ready(Box::new(f)));
        }

        unreachable!();
    }
}

impl FileSystem for AppFileSystem {
    type File = AppFile;

    fn open(&self, filename: &str) -> FileFuture {
        let f = uni_app::fs::FileSystem::open(filename).map_err(|_| FileIoError::NoSuchFile);

        Box::new(AppFileReader(match f {
            Err(err) => Err(err),
            Ok(file) => Ok(AppFile(filename.into(), file)),
        }))
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
