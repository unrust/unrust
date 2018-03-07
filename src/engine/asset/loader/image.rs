use engine::asset::loader::{Loadable, Loader};
use engine::asset::{AssetError, AssetResult, AssetSystem, File, FileFuture};
use image::RgbaImage;
use image::png;
use image;
use image::ImageDecoder;
use uni_app;

use futures::prelude::*;
use std::io;

pub struct ImageLoader {}

impl Loader<RgbaImage> for ImageLoader {
    fn load<A>(_asys: A, mut _file: Box<File>) -> AssetResult<RgbaImage>
    where
        A: AssetSystem + Clone,
    {
        unreachable!();
    }
}

struct ImageBufferLineSteam<T> {
    codec: T,
    ctx: ImageContext,
    read_bytes: usize,
    row_len: usize,
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
        if self.read_bytes > READ_BYTES_YIELD_COUNT {
            self.read_bytes -= READ_BYTES_YIELD_COUNT;
            return Ok(Async::NotReady);
        }

        let mut buffer = vec![0; self.row_len];
        self.read_bytes += self.row_len;

        match self.codec.read_scanline(&mut buffer) {
            Err(image::ImageError::ImageEnd) => Ok(Async::Ready(None)),
            Ok(_) => Ok(Async::Ready(Some(buffer))),
            Err(e) => Err(AssetError::InvalidFormat {
                path: self.ctx.file_name.clone(),
                len: self.ctx.orig_len,
                reason: format!("{:?}", e),
            }),
        }
    }
}

fn image_line_stream<T>(
    codec: T,
    ctx: ImageContext,
    row_len: usize,
) -> Box<Stream<Item = Vec<u8>, Error = AssetError>>
where
    T: image::ImageDecoder + 'static,
{
    Box::new(ImageBufferLineSteam {
        codec: codec,
        ctx: ctx,
        read_bytes: 0,
        row_len: row_len,
    })
}

#[derive(Clone)]
struct ImageContext {
    w: u32,
    h: u32,
    color: image::ColorType,
    file_name: String,
    orig_len: usize,
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
            let orig_len = whole_buf.len();

            let format = image::guess_format(&whole_buf).map_err(|e| AssetError::InvalidFormat {
                path: file_name.clone(),
                len: orig_len,
                reason: format!("{:?}", e),
            })?;

            // TODO: support other format
            assert!(format == image::ImageFormat::PNG);

            let mut codec = png::PNGDecoder::new(io::Cursor::new(whole_buf));

            let color = codec.colortype().map_err(|e| AssetError::InvalidFormat {
                path: file_name.clone(),
                len: orig_len,
                reason: format!("{:?}", e),
            })?;

            let (w, h) = codec.dimensions().map_err(|e| AssetError::InvalidFormat {
                path: file_name.clone(),
                len: orig_len,
                reason: format!("{:?}", e),
            })?;

            let ctx = ImageContext {
                w,
                h,
                color,
                orig_len,
                file_name: file_name.clone(),
            };

            let row_len = codec.row_len().map_err(|e| AssetError::InvalidFormat {
                path: ctx.file_name.clone(),
                len: ctx.orig_len,
                reason: format!("{:?}", e),
            })?;

            let stream = image_line_stream(codec, ctx.clone(), row_len);
            let buff_future = stream.fold(Vec::new(), |mut acc, mut buffer| {
                acc.append(&mut buffer);
                Ok(acc)
            });

            Ok(buff_future.map(move |buf| match color {
                image::ColorType::RGBA(8) | image::ColorType::RGB(8) => Ok((buf, ctx)),
                _ => Err(AssetError::InvalidFormat {
                    path: file_name.clone(),
                    len: orig_len,
                    reason: format!("{:?}", image::ImageError::UnsupportedColor(color)),
                }),
            }))
        });

        let decoded = decoded.flatten();

        let img = decoded.and_then(move |r| match r {
            Ok((decoded, ctx)) => {
                let t = uni_app::now();

                let img = match ctx.color {
                    image::ColorType::RGBA(_) => {
                        image::ImageBuffer::from_raw(ctx.w, ctx.h, decoded)
                            .map(image::DynamicImage::ImageRgba8)
                    }
                    image::ColorType::RGB(_) => image::ImageBuffer::from_raw(ctx.w, ctx.h, decoded)
                        .map(image::DynamicImage::ImageRgb8),
                    _ => unreachable!(),
                };

                match img {
                    Some(img) => {
                        let rgba = img.to_rgba();

                        uni_app::App::print(format!(
                            "image {} loading time : {}\n",
                            &ctx.file_name,
                            (uni_app::now() - t)
                        ));

                        Ok(rgba)
                    }

                    None => Err(AssetError::InvalidFormat {
                        path: ctx.file_name.clone(),
                        len: ctx.orig_len,
                        reason: format!("{:?}", image::ImageError::DimensionError),
                    }),
                }
            }
            Err(e) => Err(e),
        });

        // futurize
        Box::new(img)
    }
}
