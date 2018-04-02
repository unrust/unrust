use engine::asset::loader::{Loadable, Loader};
use engine::asset::{AssetError, AssetResult, AssetSystem, File, FileFuture};
use engine::render::{PreprocessedShaderCode, Shader, ShaderKind, ShaderKindFs, ShaderKindProvider,
                     ShaderKindVs};

use std::str;
use std::marker::PhantomData;
use futures::Future;
use futures::{future, Async};
use std::collections::HashMap;

use uni_glsl::preprocessor::PreprocessError;

pub struct ShaderLoader<T: ShaderKindProvider> {
    phantom: PhantomData<T>,
}

impl<T> Loader<Shader<T>> for ShaderLoader<T>
where
    T: ShaderKindProvider,
{
    fn load<A>(_asys: A, mut file: Box<File>) -> AssetResult<Shader<T>> {
        let buf = file.read_binary()
            .map_err(|_| AssetError::ReadBufferFail(file.name()))?;
        let s = str::from_utf8(&buf).map_err(|e| AssetError::InvalidFormat {
            path: file.name(),
            len: buf.len(),
            reason: format!("{:?}", e),
        })?;

        let code =
            PreprocessedShaderCode::new(T::kind(), &file.name(), s, &HashMap::new()).unwrap();
        Ok(Shader::<T>::from_preprocessed(&file.name(), code))
    }
}

struct PreprocessedShaderCodeFuture<A>
where
    A: AssetSystem + Clone + 'static,
{
    filename: String,
    source: String,
    kind: ShaderKind,
    loading_files: HashMap<String, FileFuture>,
    extern_files: HashMap<String, String>,
    asys: A,
}

impl<A> PreprocessedShaderCodeFuture<A>
where
    A: AssetSystem + Clone + 'static,
{
    fn update_externs(&mut self) -> Result<(), PreprocessError> {
        let old: Vec<_> = self.loading_files.drain().collect();

        for (k, mut v) in old {
            let r = v.poll();

            match r {
                Ok(Async::NotReady) => {
                    self.loading_files.insert(k, v);
                }
                Ok(Async::Ready(mut f)) => {
                    let buf = f.read_binary().map_err(|e| {
                        PreprocessError::MissingFile(format!(
                            "Cannot read buffer for extern file: {}, reason: {:?}",
                            f.name().clone(),
                            e
                        ))
                    })?;

                    let s = str::from_utf8(&buf).map_err(|e| {
                        PreprocessError::ParseError(format!(
                            "Cannot parse as string for extern file: {}, reason: {:?}",
                            self.filename, e
                        ))
                    })?;

                    self.extern_files.insert(k, s.to_owned());
                }
                Err(e) => {
                    return Err(PreprocessError::MissingFile(format!(
                        "Cannot open extern file: {}, reason: {:?}",
                        k, e
                    )));
                }
            }
        }

        Ok(())
    }

    fn parse(&self) -> Result<PreprocessedShaderCode, PreprocessError> {
        return PreprocessedShaderCode::new(
            self.kind,
            &self.filename,
            &self.source,
            &self.extern_files,
        );
    }
}

impl<A> Future for PreprocessedShaderCodeFuture<A>
where
    A: AssetSystem + Clone + 'static,
{
    type Item = PreprocessedShaderCode;
    type Error = PreprocessError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        self.update_externs()?;

        let r = self.parse();

        match r {
            Ok(pcode) => Ok(Async::Ready(pcode)),
            Err(PreprocessError::MissingFile(s)) => {
                if let None = self.loading_files.get(&s) {
                    self.loading_files.insert(s.clone(), self.asys.new_file(&s));
                }

                // Try to fetch the missing files.
                Ok(Async::NotReady)
            }
            Err(e) => Err(e),
        }
    }
}

fn make_preprocessor_future<A>(
    asys: A,
    filename: String,
    buf: Vec<u8>,
    kind: ShaderKind,
) -> Box<Future<Item = PreprocessedShaderCode, Error = AssetError>>
where
    A: AssetSystem + Clone + 'static,
{
    let s = str::from_utf8(&buf).map_err(|e| AssetError::InvalidFormat {
        path: filename.clone(),
        len: buf.len(),
        reason: format!("{:?}", e),
    });

    if let Err(e) = s {
        return Box::new(future::err(e));
    }

    let pcode_future = PreprocessedShaderCodeFuture {
        filename: filename.clone(),
        source: s.unwrap().to_owned(),
        kind: kind,
        asys: asys,
        loading_files: HashMap::new(),
        extern_files: HashMap::new(),
    };

    Box::new(pcode_future.map_err(move |e| AssetError::InvalidFormat {
        path: filename,
        len: buf.len(),
        reason: format!("{:?}", e),
    }))
}

impl<T: ShaderKindProvider> Loadable for Shader<T> {
    type Loader = ShaderLoader<T>;

    fn load_future<A>(asys: A, f0: FileFuture) -> Box<Future<Item = Self, Error = AssetError>>
    where
        Self: 'static,
        A: AssetSystem + Clone + 'static,
    {
        let pcode = f0.then(move |r| {
            let mut file = r.map_err(|e| AssetError::FileIoError(e))?;

            let buf = file.read_binary()
                .map_err(|_| AssetError::ReadBufferFail(file.name()))?;

            let pcode = make_preprocessor_future(asys.clone(), file.name(), buf, T::kind());

            // attach the filename to the future
            let filename = file.name().clone();
            let pcode = pcode.map(|r| (r, filename));

            Ok(pcode)
        });

        let final_code = pcode.and_then(|r| r);

        // futurize
        Box::new(final_code.and_then(|(pcode, filename)| {
            let shader = Shader::<T>::from_preprocessed(&filename, pcode);
            Ok(shader)
        }))
    }
}

pub type ShaderVSLoader = ShaderLoader<ShaderKindVs>;
pub type ShaderFSLoader = ShaderLoader<ShaderKindFs>;
