/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2025 Violet Leonard */

use std::{convert::TryInto, fs::File, io::Read, mem, path::Path, rc::Rc};

use crate::platforms::opengl;

use opengl::{
    context::{
        bindings::{
            types::{GLenum, GLint, GLsizei},
            COMPRESSED_TEXTURE_FORMATS, NUM_COMPRESSED_TEXTURE_FORMATS,
            UNSIGNED_BYTE,
        },
        short_consts::{
            COMPRESSED_RGBA_S3TC_DXT1_EXT, COMPRESSED_RGBA_S3TC_DXT3_EXT,
            COMPRESSED_RGBA_S3TC_DXT5_EXT, COMPRESSED_RGB_S3TC_DXT1_EXT, RGBA,
        },
    },
    renderer::{UvRect, UvRectValues},
    OpenGlBindings, PopulateTexture, PopulateTextureUtil, Texture,
    TextureSize,
};

pub trait LoadDds {
    fn load_dds(self) -> Texture;
}

impl<T> LoadDds for T
where
    T: 'static + AsRef<Path>,
{
    fn load_dds(self) -> Texture {
        Texture::new(Rc::new(Populator { path: self }))
    }
}

struct Populator<T> {
    path: T,
}

impl<T> std::fmt::Debug for Populator<T>
where
    T: AsRef<Path>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadDdsPopulator")
            .field("path", &self.path.as_ref())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
enum Fmt {
    Bc1Opaque = COMPRESSED_RGB_S3TC_DXT1_EXT,
    Bc1 = COMPRESSED_RGBA_S3TC_DXT1_EXT,
    Bc2 = COMPRESSED_RGBA_S3TC_DXT3_EXT,
    Bc3 = COMPRESSED_RGBA_S3TC_DXT5_EXT,
}

impl<T> PopulateTexture for Populator<T>
where
    T: AsRef<Path>,
{
    fn populate(
        &self,
        gl: &OpenGlBindings,
        target: GLenum,
    ) -> Result<TextureSize, String> {
        let mut file = File::open(&self.path).map_err(|e| e.to_string())?;
        let mut header = vec![0_u8; 128];
        file.read_exact(&mut header).map_err(|e| e.to_string())?;
        let info = parse_dds_header(&header)?;
        let short_width: u16 =
            info.width.try_into().map_err(|_| "image was too wide")?;
        let short_height: u16 =
            info.height.try_into().map_err(|_| "image was too tall")?;
        let texture_width: u16 = info
            .width
            .next_power_of_two()
            .try_into()
            .map_err(|_| "image was too wide")?;
        let texture_height: u16 = info
            .height
            .next_power_of_two()
            .try_into()
            .map_err(|_| "image was too tall")?;
        let fmt = match &info.four_cc.to_le_bytes() {
            b"DXT1" => Fmt::Bc1,
            b"DXT3" => Fmt::Bc2,
            b"DXT5" => Fmt::Bc3,
            b"DX10" => {
                let dx10_header = &mut header[..20];
                file.read_exact(dx10_header).map_err(|e| e.to_string())?;
                let dx10_info = parse_dds_header_dx10(dx10_header);
                let alpha_mode = dx10_info.misc_flags2 & 0b111;
                if dx10_info.resource_dimension != 3 {
                    return Err("only 2D textures are supported".into());
                }
                let fmt = match dx10_info.dxgi_format {
                    70..=72 if alpha_mode == 3 => Fmt::Bc1Opaque,
                    70..=72 => Fmt::Bc1,
                    73..=75 => Fmt::Bc2,
                    76..=78 => Fmt::Bc3,
                    ..70 | 79.. => {
                        return Err(
                            "DDS texture data was in an unsupported format"
                                .into(),
                        );
                    }
                };
                #[cfg(debug_assertions)]
                {
                    if dx10_info.array_size != 1 {
                        eprintln!("(warning) only the first texture of this DDS texture array will be loaded");
                    }
                    let known_alpha_modes = if fmt == Fmt::Bc1 {
                        &[0, 1, 2, 3][..]
                    } else {
                        &[0, 1, 3]
                    };
                    if !known_alpha_modes.contains(&alpha_mode) {
                        eprintln!("(warning) this DDS texture did not have a recognized alpha mode");
                    }
                }
                fmt
            }
            _ => {
                return Err(
                    "DDS texture data was in an unsupported format".into()
                );
            }
        };
        check_data_len(
            fmt,
            short_width,
            short_height,
            info.pitch_or_data_len,
        )?;
        let short_fmt = fmt as u16;
        if !get_compressed_texture_formats(gl).contains(&short_fmt.into()) {
            return Err(
                "gpu did not support this kind of compressed texture".into()
            );
        }
        let gl_data_len: GLsizei = info
            .pitch_or_data_len
            .try_into()
            .map_err(|_| "image was too big")?;
        let mut data = header; // re-use vec
        data.resize(
            info.pitch_or_data_len
                .try_into()
                .expect("usize is at least 32-bit"),
            0,
        );
        file.read_exact(&mut data).map_err(|e| e.to_string())?;
        match fmt {
            Fmt::Bc1Opaque | Fmt::Bc1 => flip_bc1(&mut data, short_width),
            Fmt::Bc2 => flip_bc2(&mut data, short_width),
            Fmt::Bc3 => flip_bc3(&mut data, short_width),
        }
        if texture_width == short_width && texture_height == short_height {
            unsafe {
                gl.CompressedTexImage2D(
                    target,
                    0,
                    short_fmt.into(),
                    short_width.into(),
                    short_height.into(),
                    0,
                    gl_data_len,
                    data.as_ptr().cast(),
                );
            }
        } else {
            unsafe {
                gl.TexImage2D(
                    target,
                    0,
                    short_fmt.into(),
                    texture_width.into(),
                    texture_height.into(),
                    0,
                    RGBA.into(),
                    UNSIGNED_BYTE,
                    std::ptr::null(),
                );
                gl.CompressedTexSubImage2D(
                    target,
                    0,
                    0,
                    0,
                    short_width.into(),
                    short_height.into(),
                    short_fmt.into(),
                    gl_data_len,
                    data.as_ptr().cast(),
                );
            }
        }
        PopulateTextureUtil::default_params(gl, target);
        Ok(TextureSize {
            default_rect: UvRect::U16(UvRectValues {
                left: 0,
                right: short_width,
                bottom: 0,
                top: short_height,
            }),
            uv_scale: [texture_width, texture_height],
            is_sdf: false,
        })
    }

    fn texture_key(&self) -> &[u8] {
        self.path.as_ref().as_os_str().as_encoded_bytes()
    }

    fn debug(&self) -> &dyn std::fmt::Debug {
        self
    }
}

fn get_compressed_texture_formats(gl: &OpenGlBindings) -> Vec<GLint> {
    let mut nctf: GLint = 0;
    unsafe {
        gl.GetIntegerv(NUM_COMPRESSED_TEXTURE_FORMATS, &raw mut nctf);
    }
    let nctf: usize = nctf.try_into().expect(
        "glGetIntegerv returned a number which is invalid for use as a size",
    );
    let zero: GLint = 0;
    let mut ctfs = vec![zero; nctf];
    unsafe { gl.GetIntegerv(COMPRESSED_TEXTURE_FORMATS, ctfs.as_mut_ptr()) }
    ctfs
}

struct DdsHeader {
    width: u32,
    height: u32,
    pitch_or_data_len: u32,
    #[expect(unused)]
    mipmap_count: u32,
    four_cc: u32,
}

fn parse_dds_header(header: &[u8]) -> Result<DdsHeader, &'static str> {
    let (header_words, tail) = header.as_chunks();
    debug_assert_eq!(header_words.len(), 32);
    debug_assert_eq!(tail.len(), 0);
    if header_words[0] != *b"DDS " {
        return Err("not a dds file");
    }
    if u32::from_le_bytes(header_words[1]) != 124 {
        return Err("not a dds file");
    }
    //let flags = u32::from_le_bytes(header_words[2]);
    let height = u32::from_le_bytes(header_words[3]);
    let width = u32::from_le_bytes(header_words[4]);
    let pitch_or_data_len = u32::from_le_bytes(header_words[5]);
    //let depth = u32::from_le_bytes(header_words[6]);
    let mipmap_count = u32::from_le_bytes(header_words[7]);
    // header_words[8..19] are reserved
    if u32::from_le_bytes(header_words[19]) != 32 {
        return Err("not a dds file");
    }
    let pf_flags = u32::from_le_bytes(header_words[20]);
    if pf_flags != 0x4 {
        return Err("unsupported texture format");
    }
    let four_cc = u32::from_le_bytes(header_words[21]);
    Ok(DdsHeader {
        width,
        height,
        pitch_or_data_len,
        mipmap_count,
        four_cc,
    })
}

struct DdsHeaderDxt10 {
    dxgi_format: u32,
    resource_dimension: u32,
    #[expect(unused)]
    misc_flag: u32,
    array_size: u32,
    misc_flags2: u32,
}

fn parse_dds_header_dx10(header: &[u8]) -> DdsHeaderDxt10 {
    use std::convert::TryFrom;

    let (header_words, tail) = header.as_chunks();
    debug_assert_eq!(tail.len(), 0);
    let header_words = <[_; 5]>::try_from(header_words)
        .expect("parse_dds_header_dx10 should be called with a 20-byte slice");
    let [dxgi_format, resource_dimension, misc_flag, array_size, misc_flags2] =
        header_words.map(u32::from_le_bytes);
    DdsHeaderDxt10 {
        dxgi_format,
        resource_dimension,
        misc_flag,
        array_size,
        misc_flags2,
    }
}

fn check_data_len(
    fmt: Fmt,
    width: u16,
    height: u16,
    data_len: u32,
) -> Result<(), &'static str> {
    #[cfg(debug_assertions)]
    {
        let num_blocks =
            (width.next_multiple_of(4) / 4) * (height.next_multiple_of(4) / 4);
        let block_size = match fmt {
            Fmt::Bc1Opaque | Fmt::Bc1 => 8,
            Fmt::Bc2 | Fmt::Bc3 => 16,
        };
        if u32::from(num_blocks) * block_size != data_len {
            return Err("DDS data len did not match the expected length");
        }
    }
    Ok(())
}

fn flip_bc1(data: &mut [u8], width: u16) {
    let blocks_wide = width.div_ceil(4);
    // reverse the rows of blocks
    let mut rows_of_blocks =
        data.chunks_exact_mut(usize::from(blocks_wide) * 8);
    while let [Some(a), Some(b)] =
        [rows_of_blocks.next(), rows_of_blocks.next_back()]
    {
        a.swap_with_slice(b);
    }
    // reverse the data in each block
    let (half_blocks, _tail) = data.as_chunks_mut();
    for lookup_bytes in half_blocks.iter_mut().skip(1).step_by(2) {
        // this is surprisingly faster than `lookup_bytes.reverse()`
        *lookup_bytes =
            u32::from_ne_bytes(*lookup_bytes).swap_bytes().to_ne_bytes();
    }
}

fn flip_bc2(data: &mut [u8], width: u16) {
    let blocks_wide = width.div_ceil(4);
    // reverse the rows of blocks
    let mut rows_of_blocks =
        data.chunks_exact_mut(usize::from(blocks_wide) * 16);
    while let [Some(a), Some(b)] =
        [rows_of_blocks.next(), rows_of_blocks.next_back()]
    {
        a.swap_with_slice(b);
    }
    // reverse the data in each block
    let (blocks, _tail) = data.as_chunks_mut::<16>();
    for block in blocks {
        // each row of alpha data is 2 bytes
        let [a, b, c, d, e, f, g, h, color @ ..] = block;
        mem::swap(a, g);
        mem::swap(b, h);
        mem::swap(c, e);
        mem::swap(d, f);
        let [_, _, _, _, lookup_bytes @ ..] = color;
        *lookup_bytes =
            u32::from_ne_bytes(*lookup_bytes).swap_bytes().to_ne_bytes();
    }
}

fn flip_bc3(data: &mut [u8], width: u16) {
    let blocks_wide = width.div_ceil(4);
    // reverse the rows of blocks
    let mut rows_of_blocks =
        data.chunks_exact_mut(usize::from(blocks_wide) * 16);
    while let [Some(a), Some(b)] =
        [rows_of_blocks.next(), rows_of_blocks.next_back()]
    {
        a.swap_with_slice(b);
    }
    // reverse the data in each block
    let (blocks, _tail) = data.as_chunks_mut();
    for block in blocks {
        // it might be a little odd to do this in little-endian
        let bits = u128::from_le_bytes(*block);
        // the alpha indices are 3 bits. 3 bits * 4 pixels/row = 12 bits/row so
        // each row is 3 nibbles
        let alpha_row_a = (bits >> 16_u8) & 0xFFF;
        let alpha_row_b = (bits >> 28_u8) & 0xFFF;
        let alpha_row_c = (bits >> 40_u8) & 0xFFF;
        let alpha_row_d = (bits >> 52_u8) & 0xFFF;
        // in little endian the color indices are the highest 32 bits
        let color_lookup = (bits >> 96_u8) as u32;
        // palette bits are passed through unchanged
        let unchanged = bits & 0x00000000FFFFFFFF000000000000FFFF;
        // using opposite shift values reverses the rows
        let alpha_row_a = alpha_row_a << 52_u8;
        let alpha_row_b = alpha_row_b << 40_u8;
        let alpha_row_c = alpha_row_c << 28_u8;
        let alpha_row_d = alpha_row_d << 16_u8;
        // use the specialized swap_bytes for the color indices
        let color_lookup = u128::from(color_lookup.swap_bytes()) << 96_u8;
        // merge everything back together
        *block = (unchanged
            | alpha_row_a
            | alpha_row_b
            | alpha_row_c
            | alpha_row_d
            | color_lookup)
            .to_le_bytes();
    }
}
