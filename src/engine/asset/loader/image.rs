use engine::asset::loader::{Loadable, Loader};
use engine::asset::{AssetError, AssetResult, AssetSystem, File, FileFuture};
use engine::TextureImage;
use image::png;
use image::tga;
use image;

use futures::prelude::*;
use futures::future;

use std::io;
use std::fmt::Debug;
use uni_app;
use std::path::Path;

use super::dds::{DDSFormat, DDSReader};

pub struct ImageLoader {}

impl Loader<TextureImage> for ImageLoader {
    fn load<A>(_asys: A, mut _file: Box<File>) -> AssetResult<TextureImage>
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

    max_read_bytes: usize,
    read_timer: f64,
}

// the number of bytes read for yield
const MIN_READ_BYTES_YIELD_COUNT: usize = 1024 * 100;
const ONE_FRAME: f64 = 1.0 / (60.0);

impl<T> Stream for ImageBufferLineSteam<T>
where
    T: image::ImageDecoder,
{
    type Item = (Vec<u8>, u32);
    type Error = AssetError;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        // first time initialize timer
        if self.read_timer == 0.0 {
            self.read_timer = uni_app::now();
        }

        // The main magic is in here
        // We just return an Async::NotReady to 'yield' the execution
        // if the timer is smaller than one frame,
        // we double up the read_bytes count,
        // so in theory it will reach to yield per frame
        if self.read_bytes > self.max_read_bytes {
            self.read_bytes -= self.max_read_bytes;
            let now = uni_app::now();
            if now - self.read_timer < ONE_FRAME {
                self.max_read_bytes *= 2;
                self.read_timer = now;
            }

            return Ok(Async::NotReady);
        }

        let mut buffer = vec![0; self.ctx.row_len];
        self.read_bytes += self.ctx.row_len;

        match self.codec.read_scanline(&mut buffer) {
            Err(image::ImageError::ImageEnd) => Ok(Async::Ready(None)),
            Ok(row_index) => Ok(Async::Ready(Some((buffer, row_index)))),
            Err(e) => Err(make_invalid_format(&self.ctx.info, e)),
        }
    }
}

fn image_rows_stream<T>(
    codec: T,
    ctx: ImageContext,
) -> Box<Stream<Item = (Vec<u8>, u32), Error = AssetError>>
where
    T: image::ImageDecoder + 'static,
{
    Box::new(ImageBufferLineSteam {
        codec: codec,
        read_bytes: 0,
        ctx: ctx,
        max_read_bytes: MIN_READ_BYTES_YIELD_COUNT,
        read_timer: 0.0,
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

fn guess_format(
    info: &ImageFileInfo,
    buf: &Vec<u8>,
) -> Result<image::ImageFormat, image::ImageError> {
    let path = Path::new(&info.file_name);
    let ext = path.extension();

    if let Some(exts) = ext.and_then(|s| s.to_str()) {
        if exts.to_string().to_lowercase() == "tga" {
            return Ok(image::ImageFormat::TGA);
        }
    }

    // image::guess_format do NOT support tga
    let format = image::guess_format(&buf)?;

    return Ok(format);
}

enum ImageCodec {
    NoMatch,
    Png(png::PNGDecoder<io::Cursor<Vec<u8>>>),
    Tga(tga::TGADecoder<io::Cursor<Vec<u8>>>),
}

fn new_img_context_from_decoder<T>(
    decoder: &mut T,
    info: ImageFileInfo,
) -> Result<ImageContext, image::ImageError>
where
    T: image::ImageDecoder,
{
    let color = decoder.colortype()?;
    let (w, h) = decoder.dimensions()?;
    let row_len = decoder.row_len()?;

    Ok(ImageContext {
        w,
        h,
        color,
        row_len,
        info,
    })
}

fn new_img_context(
    buf: Vec<u8>,
    info: ImageFileInfo,
) -> Result<(ImageContext, ImageCodec), image::ImageError> {
    let format = guess_format(&info, &buf)?;

    // TODO: support other format
    let mut codec = match format {
        image::ImageFormat::PNG => ImageCodec::Png(png::PNGDecoder::new(io::Cursor::new(buf))),
        image::ImageFormat::TGA => ImageCodec::Tga(tga::TGADecoder::new(io::Cursor::new(buf))),
        _ => ImageCodec::NoMatch,
    };

    if let ImageCodec::NoMatch = codec {
        return Err(image::ImageError::UnsupportedError(
            "Not support image type in unrust image loader".to_string(),
        ));
    }

    let ctx = match codec {
        ImageCodec::Png(ref mut decoder) => new_img_context_from_decoder(decoder, info)?,
        ImageCodec::Tga(ref mut decoder) => new_img_context_from_decoder(decoder, info)?,
        _ => unreachable!(),
    };

    Ok((ctx, codec))
}

fn load_future_uncompressed<T>(img_buf: T) -> Box<Future<Item = TextureImage, Error = AssetError>>
where
    T: Future<Item = (Vec<u8>, String), Error = AssetError> + 'static,
{
    let decoded = img_buf.and_then(|(whole_buf, file_name)| {
        let info = ImageFileInfo {
            file_name: file_name.clone(),
            orig_len: whole_buf.len(),
        };

        let (ctx, codec) =
            new_img_context(whole_buf, info.clone()).map_err(|e| AssetError::InvalidFormat {
                path: info.file_name.clone(),
                len: info.orig_len,
                reason: format!("{:?}", e),
            })?;

        let mut iter = 1..;
        let f0: Box<FnMut(u32) -> u32> = Box::new(|row_index: u32| -> u32 { row_index });
        let f1: Box<FnMut(u32) -> u32> = Box::new(move |_: u32| -> u32 { iter.next().unwrap() });

        let (mut indexer, stream) = match codec {
            ImageCodec::Tga(decoder) => (f0, image_rows_stream(decoder, ctx.clone())),
            ImageCodec::Png(decoder) => (f1, image_rows_stream(decoder, ctx.clone())),
            _ => unreachable!(),
        };

        use std;
        let num_rows = ctx.h as usize;
        let row_len = ctx.row_len;
        let rows = std::iter::repeat(vec![]).take(num_rows).collect::<Vec<_>>();
        let err_filename = file_name.clone();

        let buff_future = stream.fold(rows, move |mut acc, (buffer, row_index)| {
            let idx = indexer(row_index) as usize - 1;

            debug_assert!(
                (idx as usize) < num_rows,
                format!("idx is wrong: {:?} {:?} {}", idx, num_rows, &err_filename)
            );
            debug_assert!(buffer.len() == row_len as usize);

            // we flip the image vertically because opengl is use bottom left as (0,0)
            acc[idx] = buffer;
            Ok(acc)
        });

        Ok(buff_future.map(move |buf| match ctx.color {
            image::ColorType::RGBA(8) | image::ColorType::RGB(8) | image::ColorType::Gray(8) => {
                Ok((buf, ctx))
            }
            _ => Err(make_invalid_format(
                &ctx.info,
                image::ImageError::UnsupportedColor(ctx.color),
            )),
        }))
    });

    let decoded = decoded.flatten().and_then(|r| r);

    let img = decoded.and_then(move |(decoded_array, ctx)| {
        assert!(decoded_array.len() == (ctx.h as usize));
        let mut decoded = Vec::new();
        for mut row in decoded_array.into_iter() {
            assert!(
                row.len() == ctx.row_len,
                format!("row.len = {}, ctx.row_len = {}", row.len(), ctx.row_len)
            );
            decoded.append(&mut row);
        }
        let decoded_len = decoded.len();

        let img = match ctx.color {
            image::ColorType::RGBA(_) => {
                image::ImageBuffer::from_raw(ctx.w, ctx.h, decoded).map(TextureImage::Rgba)
            }
            image::ColorType::RGB(_) => {
                image::ImageBuffer::from_raw(ctx.w, ctx.h, decoded).map(TextureImage::Rgb)
            }
            image::ColorType::Gray(_) => {
                let raw = image::ImageBuffer::<image::Luma<u8>, _>::from_raw(ctx.w, ctx.h, decoded);
                debug_assert!(raw.is_some());

                raw.map(image::DynamicImage::ImageLuma8)
                    .map(|img| TextureImage::Rgb(img.to_rgb()))
            }

            _ => unreachable!(),
        };

        match img {
            Some(img) => Ok(img),
            None => Err(make_invalid_format(
                &ctx.info,
                image::ImageError::UnsupportedError(format!(
                    "Unknown ctx.w = {}, ctx.h = {} decode.len = {}, row_len = {}",
                    ctx.w, ctx.h, decoded_len, ctx.row_len
                )),
            )),
        }
    });

    // futurize
    Box::new(img)
}

static DDS_MAGIC_BYTES: &'static [u8] = b"DDS ";

fn load_future_dds<T>(img_buf: T) -> Box<Future<Item = TextureImage, Error = AssetError>>
where
    T: Future<Item = (Vec<u8>, String), Error = AssetError> + 'static,
{
    let img = img_buf.and_then(|(whole_buf, file_name)| {
        DDSReader::read(whole_buf, &file_name).map(|dds| match dds.format {
            DDSFormat::DXT1 => TextureImage::DXT1(dds),
            DDSFormat::DXT5 => TextureImage::DXT5(dds),
        })
    });

    Box::new(img)
}

impl Loadable for TextureImage {
    type Loader = ImageLoader;

    fn load_future<A>(_asys: A, objfile: FileFuture) -> Box<Future<Item = Self, Error = AssetError>>
    where
        Self: 'static,
        A: AssetSystem + Clone + 'static,
    {
        let img_buf = objfile.then(move |mut f| match f {
            Ok(ref mut file) => {
                let buf = file.read_binary()
                    .map_err(|_| AssetError::ReadBufferFail(file.name()))?;
                Ok((buf, file.name()))
            }
            Err(e) => Err(AssetError::FileIoError(e)),
        });

        Box::new(img_buf.and_then(|(whole_buf, file_name)| {
            if whole_buf.starts_with(DDS_MAGIC_BYTES) {
                return load_future_dds(future::result(Ok((whole_buf, file_name))));
            }

            load_future_uncompressed(future::result(Ok((whole_buf, file_name))))
        }))
    }
}
