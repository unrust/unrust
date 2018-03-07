use engine::asset::loader::{Loadable, Loader};
use engine::asset::{AssetError, AssetResult, AssetSystem, File, FileFuture};
use image::RgbaImage;
use image::png;
use image;
use image::ImageDecoder;
use uni_app;

use futures::prelude::*;
use std::io;
use std::fmt::Debug;

pub struct ImageLoader {}

impl Loader<RgbaImage> for ImageLoader {
    fn load<A>(_asys: A, mut _file: Box<File>) -> AssetResult<RgbaImage>
    where
        A: AssetSystem + Clone,
    {
        unreachable!();
    }
}

fn make_invalid_format<E: Debug>(info: &ImageFileInfo, e: E) -> AssetError {
    AssetError::InvalidFormat {
        path: info.file_name.clone(),
        len: info.orig_len,
        reason: format!("{:?}", e),
    }
}

struct ImageBufferLineSteam<T> {
    codec: T,
    ctx: ImageContext,
    read_bytes: usize,
}

// the number of bytes read for yield
const READ_BYTES_YIELD_COUNT: usize = 1024 * 100;

impl<T> Stream for ImageBufferLineSteam<T>
where
    T: image::ImageDecoder,
{
    type Item = Vec<u8>;
    type Error = AssetError;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        // The main magic is in here
        // We just return an Async::NotReady to 'yield' the execution
        if self.read_bytes > READ_BYTES_YIELD_COUNT {
            self.read_bytes -= READ_BYTES_YIELD_COUNT;
            return Ok(Async::NotReady);
        }

        let mut buffer = vec![0; self.ctx.row_len];
        self.read_bytes += self.ctx.row_len;

        match self.codec.read_scanline(&mut buffer) {
            Err(image::ImageError::ImageEnd) => Ok(Async::Ready(None)),
            Ok(_) => Ok(Async::Ready(Some(buffer))),
            Err(e) => Err(make_invalid_format(&self.ctx.info, e)),
        }
    }
}

fn image_rows_stream<T>(
    codec: T,
    ctx: ImageContext,
) -> Box<Stream<Item = Vec<u8>, Error = AssetError>>
where
    T: image::ImageDecoder + 'static,
{
    Box::new(ImageBufferLineSteam {
        codec: codec,
        read_bytes: 0,
        ctx: ctx,
    })
}

#[derive(Clone)]
struct ImageFileInfo {
    file_name: String,
    orig_len: usize,
}

#[derive(Clone)]
struct ImageContext {
    w: u32,
    h: u32,
    color: image::ColorType,
    info: ImageFileInfo,
    row_len: usize,
}

fn new_img_context(
    buf: Vec<u8>,
    info: ImageFileInfo,
) -> Result<(ImageContext, png::PNGDecoder<io::Cursor<Vec<u8>>>), image::ImageError> {
    let format = image::guess_format(&buf)?;

    // TODO: support other format
    assert!(format == image::ImageFormat::PNG);

    let mut codec = png::PNGDecoder::new(io::Cursor::new(buf));

    let color = codec.colortype()?;
    let (w, h) = codec.dimensions()?;
    let row_len = codec.row_len()?;

    Ok((
        ImageContext {
            w,
            h,
            color,
            row_len,
            info,
        },
        codec,
    ))
}

impl Loadable for RgbaImage {
    type Loader = ImageLoader;

    fn load_future<A>(_asys: A, objfile: FileFuture) -> Box<Future<Item = Self, Error = AssetError>>
    where
        Self: 'static,
        A: AssetSystem + Clone + 'static,
    {
        let img_buf = {
            objfile.then(move |mut f| match f {
                Ok(ref mut file) => {
                    let buf = file.read_binary()
                        .map_err(|_| AssetError::ReadBufferFail(file.name()))?;
                    Ok((buf, file.name()))
                }
                Err(e) => Err(AssetError::FileIoError(e)),
            })
        };

        let decoded = img_buf.and_then(|(whole_buf, file_name)| {
            let info = ImageFileInfo {
                file_name,
                orig_len: whole_buf.len(),
            };

            let (ctx, codec) =
                new_img_context(whole_buf, info.clone()).map_err(|e| AssetError::InvalidFormat {
                    path: info.file_name.clone(),
                    len: info.orig_len,
                    reason: format!("{:?}", e),
                })?;

            let stream = image_rows_stream(codec, ctx.clone());
            let buff_future = stream.fold(Vec::new(), |mut acc, mut buffer| {
                acc.append(&mut buffer);
                Ok(acc)
            });

            Ok(buff_future.map(move |buf| match ctx.color {
                image::ColorType::RGBA(8) | image::ColorType::RGB(8) => Ok((buf, ctx)),
                _ => Err(make_invalid_format(
                    &ctx.info,
                    image::ImageError::UnsupportedColor(ctx.color),
                )),
            }))
        });

        let decoded = decoded.flatten().and_then(|r| r);

        let img = decoded.and_then(move |(decoded, ctx)| {
            let t = uni_app::now();

            let img = match ctx.color {
                image::ColorType::RGBA(_) => image::ImageBuffer::from_raw(ctx.w, ctx.h, decoded)
                    .map(image::DynamicImage::ImageRgba8),
                image::ColorType::RGB(_) => image::ImageBuffer::from_raw(ctx.w, ctx.h, decoded)
                    .map(image::DynamicImage::ImageRgb8),
                _ => unreachable!(),
            };

            match img {
                Some(img) => {
                    let rgba = img.to_rgba();
                    uni_app::App::print(format!(
                        "image {} loading time : {}\n",
                        &ctx.info.file_name,
                        (uni_app::now() - t)
                    ));

                    Ok(rgba)
                }

                None => Err(make_invalid_format(
                    &ctx.info,
                    image::ImageError::DimensionError,
                )),
            }
        });

        // futurize
        Box::new(img)
    }
}
