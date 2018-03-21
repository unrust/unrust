use engine::asset::{AssetError, AssetResult};
use std::mem;
use std::slice;
use std::io::Read;

pub struct DDSReader {}

#[derive(Debug, Clone)]
pub enum DDSFormat {
    DXT1,
    DXT5,
}

#[derive(Debug, Clone)]
pub struct DDSImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct DDS {
    pub format: DDSFormat,
    pub images: Vec<DDSImage>,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct DDSPixelFormat {
    dw_size: u32, // offset: 19
    dw_flags: u32,
    dw_four_cc: [u8; 4],
    dw_rgb_bitcount: u32,
    dw_rbit_mask: u32,
    dw_gbit_mask: u32,
    dw_bbit_mask: u32,
    dw_abit_mask: u32, // offset: 26
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct DDSHeader {
    dw_size: u32,
    dw_flags: u32,
    dw_height: u32,
    dw_width: u32,
    dw_pitchorlinearsize: u32,
    dw_depth: u32,
    dw_mipmapcount: u32, // offset: 7
    dw_reserved1: [u32; 11],
    dd_spf: DDSPixelFormat, // offset 19
    dw_caps: u32,           // offset: 27
    dw_caps2: u32,
    dw_caps3: u32,
    dw_caps4: u32,
    dw_reserved2: u32, // offset 31
}

const DDPF_FOURCC: u32 = 0x4;
const DDSD_MIPMAPCOUNT: u32 = 0x20000;

impl DDSReader {
    pub fn read(buff: Vec<u8>, file_name: &String) -> AssetResult<DDS> {
        let mut header: DDSHeader = unsafe { mem::zeroed() };

        let header_size = mem::size_of::<DDSHeader>();
        let mut buffer = &buff[4..];

        unsafe {
            let header_slice =
                slice::from_raw_parts_mut(&mut header as *mut _ as *mut u8, header_size);

            // `read_exact()` comes from `Read` impl for `&[u8]`
            buffer
                .read_exact(header_slice)
                .map_err(|_| AssetError::InvalidFormat {
                    len: buffer.len(),
                    path: file_name.clone(),
                    reason: "Invalid DDS Header Format".to_owned(),
                })?;
        }

        if header.dd_spf.dw_flags & DDPF_FOURCC == 0 {
            return Err(AssetError::InvalidFormat {
                len: buffer.len(),
                path: file_name.clone(),
                reason: "Unsupported Format, must contain a FourCC code".to_owned(),
            });
        }

        let format: DDSFormat;
        let block_bytes: u32;
        let mut mipmap_count = 1;

        match header.dd_spf.dw_four_cc {
            c if c.starts_with(b"DXT1") => {
                format = DDSFormat::DXT1;
                block_bytes = 8;
            }
            c if c.starts_with(b"DXT5") => {
                format = DDSFormat::DXT5;
                block_bytes = 16;
            }
            _ => {
                return Err(AssetError::InvalidFormat {
                    len: buffer.len(),
                    path: file_name.clone(),
                    reason: format!(
                        "Unsupported Format, only support DXT1, DXT5 (current: {:?})",
                        header.dd_spf.dw_four_cc
                    ),
                })
            }
        }

        if header.dw_flags & DDSD_MIPMAPCOUNT != 0 {
            mipmap_count = 1.max(header.dw_mipmapcount);
        }

        let mut width = header.dw_width;
        let mut height = header.dw_height;
        let mut data_offset: usize = (header.dw_size + 4) as usize;

        let mut images = Vec::new();

        for _ in 0..mipmap_count {
            let data_length = 4.max(width) / 4 * 4.max(height) / 4 * block_bytes;
            let byte_array = buff[data_offset..data_offset + data_length as usize].to_vec();

            images.push(DDSImage {
                width: width,
                height: height,
                data: byte_array,
            });

            data_offset = data_offset + (data_length as usize);
            width = width >> 1;
            height = height >> 1;
        }

        Ok(DDS { format, images })
    }
}
