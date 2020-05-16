use std::path::{Path, PathBuf};
use std::io::Write;
use std::collections::HashMap;

use rusttype::Font;

use suzy_atlas::packer::PackerNode;

#[derive(Clone, Copy, PartialEq)]
pub enum AssetSize {
    FontSize(f64),
    TextureSize(usize),
}

pub struct Settings {
    chars: Vec<char>,
    padding_ratio: f64,
    size: AssetSize,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            chars: Vec::new(),
            padding_ratio: 0.5,
            size: AssetSize::TextureSize(64),
        }
    }
}

impl Settings {
    pub fn padding_ratio(mut self, ratio: f64) -> Self {
        self.padding_ratio = ratio;
        self
    }

    pub fn size(mut self, size: AssetSize) -> Self {
        self.size = size;
        self
    }

    pub fn add_chars<I>(mut self, chars: I) -> Settings
        where I: IntoIterator<Item=char>
    {
        self.chars.extend(chars);
        self
    }

    pub fn ascii(self) -> Settings {
        self.add_chars((b'!'..=b'~').map(char::from))
    }

    pub fn latin1(self) -> Settings {
        self.ascii().add_chars((0xa1..=0xff).map(char::from))
    }
}

pub trait MaybePath {
    fn try_as_path(&self) -> Option<&Path>;
}

impl MaybePath for str {
    fn try_as_path(&self) -> Option<&Path> { Some(self.as_ref()) }
}

impl MaybePath for &str {
    fn try_as_path(&self) -> Option<&Path> { Some(self.as_ref()) }
}

impl MaybePath for () {
    fn try_as_path(&self) -> Option<&Path> { None }
}

pub struct FontFamily<A, B, C, D>
where
    A: MaybePath,
    B: MaybePath,
    C: MaybePath,
    D: MaybePath,
{
    pub normal: A,
    pub bold: B,
    pub italic: C,
    pub bold_italic: D,
}

pub fn build_fontasset<A, B, C, D, Dest>(
    ttf_family: FontFamily<A, B, C, D>,
    dest_file: Dest,
    settings: Settings,
)
where
    A: MaybePath,
    B: MaybePath,
    C: MaybePath,
    D: MaybePath,
    Dest: AsRef<Path>,
{
    fn read(path: &Path) -> Font<'static> {
        let bytes = std::fs::read(path).expect("Failed to read font file");
        Font::try_from_vec(bytes).expect("Failed to parse font data")
    }

    let fonts = (
        ttf_family.normal.try_as_path().map(read),
        ttf_family.bold.try_as_path().map(read),
        ttf_family.italic.try_as_path().map(read),
        ttf_family.bold_italic.try_as_path().map(read),
    );

    let get_size = |font: &Font<'static>| {
        get_packed_size(settings.padding_ratio, font, &settings.chars)
    };

    let sizes = (
        fonts.0.as_ref().map_or(0.0, get_size),
        fonts.1.as_ref().map_or(0.0, get_size),
        fonts.2.as_ref().map_or(0.0, get_size),
        fonts.3.as_ref().map_or(0.0, get_size),
    );

    let max = sizes.0.max(sizes.1).max(sizes.2).max(sizes.3);

    let layout = |font: &Font<'static>| {
        let packer = get_layout(
            settings.padding_ratio,
            font,
            &settings.chars,
            max,
        ).expect("Failed to pack a font that previously fit in a larger box");
        let mut flat = HashMap::new();
        packer.for_each(|x, y, ch| { flat.insert(*ch, (x / max, y / max)); });
        flat
    };

    let uniform = (
        fonts.0.as_ref().map(layout),
        fonts.1.as_ref().map(layout),
        fonts.2.as_ref().map(layout),
        fonts.3.as_ref().map(layout),
    );

    let (font_size, texture_size) = match settings.size {
        AssetSize::FontSize(font_size) => {
            let tex_size = ((font_size * max) as usize).next_power_of_two();
            ((tex_size as f64) / max, tex_size)
        },
        AssetSize::TextureSize(tex_size) => {
            ((tex_size as f64) / max, tex_size)
        },
    };

    let padding = font_size * settings.padding_ratio;

    let dest_scale = rusttype::Scale::uniform(font_size as f32);
    let ref_scale = font_size * padding;
    let ref_scale = rusttype::Scale::uniform(ref_scale as f32);

    let channels = (
        fonts.0.is_some() as usize,
        fonts.1.is_some() as usize,
        fonts.2.is_some() as usize,
        fonts.3.is_some() as usize,
    );
    let channels = channels.0 + channels.1 + channels.2 + channels.3;
    let channels = if channels == 2 { 3 } else { channels };

    let bufsize = channels * texture_size * texture_size;

    let buffer = settings.chars.iter()
        .fold(
            vec![0u8; bufsize],
            |mut buffer, ch| {
                let mut channel = 0;
                if let Some(font) = &fonts.0 {
                    let (x, y) = uniform.0.as_ref().unwrap()[&ch];
                    render_char(
                        CharToRender {
                            ch: *ch,
                            font,
                            dest_scale,
                            ref_scale,
                            x,
                            y,
                            chan: channel,
                            padding,
                        },
                        DestBuffer {
                            buffer: &mut buffer,
                            tex_size: texture_size,
                            nchans: channels,
                        },
                    );
                }
                buffer
            }
        );
        /*
        .reduce_with(
            |mut a, mut b| {
                for (dest, other) in a.iter_mut().zip(b.iter()) {
                    *dest += *other;
                }
                a
            }
        )
        .unwrap_or_else(|| vec![0u8; bufsize]);
        */
    let mut header = vec![0u8; 54];
    let palette = (0u8..=0xff).flat_map(|x| vec![x, x, x, 0x00])
        .collect::<Vec<_>>();
    let prelen = (header.len() + palette.len()) as i32;
    header[0] = b'B';
    header[1] = b'M';
    let image_size = bufsize as i32;
    let width = texture_size as i32;
    let height = texture_size as i32;
    header[0x02..0x02+4].copy_from_slice(&(image_size + prelen).to_le_bytes());
    header[0x0a..0x0a+4].copy_from_slice(&prelen.to_le_bytes());
    header[0x0e..0x0e+4].copy_from_slice(&40i32.to_le_bytes());
    header[0x12..0x12+4].copy_from_slice(&width.to_le_bytes());
    header[0x16..0x16+4].copy_from_slice(&height.to_le_bytes());
    header[0x1a..0x1a+2].copy_from_slice(&1i16.to_le_bytes());
    header[0x1c..0x1c+2].copy_from_slice(&8i16.to_le_bytes());
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open("debug_atlas.bmp").unwrap();
    file.write(&header);
    file.write(&palette);
    file.write(&buffer);
}

struct CharToRender<'a> {
    ch: char,
    font: &'a Font<'static>,
    dest_scale: rusttype::Scale,
    ref_scale: rusttype::Scale,
    x: f64,
    y: f64,
    chan: usize,
    padding: f64,
}

struct DestBuffer<'a> {
    buffer: &'a mut [u8],
    tex_size: usize,
    nchans: usize,
}

fn render_char(
    ch: CharToRender<'_>,
    mut buf: DestBuffer<'_>,
) {
    let zero = rusttype::Point { x: 0.0, y: 0.0 };
    let glyph = ch.font.glyph(ch.ch).scaled(ch.ref_scale).positioned(zero);
    let pxbb = glyph.pixel_bounding_box().expect("no pixel bounding box?");
    let bm_bufsize = (pxbb.width() * pxbb.height()) as usize;
    let pitch = pxbb.width() as usize;
    let mut bitmap = vec![false; bm_bufsize];
    glyph.draw(|x, y, v| {
        let x = x as usize;
        let y = y as usize;
        bitmap[y * pitch + x] = v >= 0.5;
    });
    let x_offset = (ch.x * (buf.tex_size as f64)).floor() as usize;
    let y_offset = (ch.y * (buf.tex_size as f64)).floor() as usize;
    let bb = ch.font.glyph(ch.ch)
        .scaled(ch.dest_scale)
        .exact_bounding_box()
        .expect("no bounding box for ?");
    let padding = ch.padding;
    let width = (bb.width() as f64 + 2.0 * padding).floor() as usize;
    let height = (bb.height() as f64 + 2.0 * padding).floor() as usize;
    let source = suzy_sdf::SourceBitmap {
        buffer: &bitmap,
        width: pxbb.width() as usize,
        height: pxbb.height() as usize,
    };
    let dest = suzy_sdf::DestImage {
        buf: suzy_sdf::DestBuffer {
            buffer: buf.buffer,
            width: buf.tex_size,
            height: buf.tex_size,
            num_channels: buf.nchans,
        },
        padding,
        channel: ch.chan,
        x_offset,
        y_offset,
        width,
        height,
    };
    suzy_sdf::render_sdf(dest, source);
    eprintln!("done rendering '{}'", ch.ch);
}

fn get_packed_size(padding_ratio: f64, font: &Font<'static>, chars: &[char])
    -> f64
{
    let scale = rusttype::Scale::uniform(1.0);
    let glyphs = font.glyphs_for(chars.iter().copied())
        .map(|glyph| glyph.scaled(scale));

    let padding = padding_ratio * 2.0;
    let mut sum_width = 0.0;
    let mut sum_height = 0.0;
    for glyph in glyphs {
        let rect = glyph.exact_bounding_box().expect("no bounding box?");
        sum_width += rect.width() + padding as f32;
        sum_height += rect.height() + padding as f32;
    }

    let mut high: f64 = sum_width.max(sum_height).into();
    high *= 1.1;
    let mut low = 0.0f64;

    loop {
        let size = (high + low) / 2.0;
        if let Some(packer) = get_layout(
            padding_ratio,
            font,
            chars,
            size,
        ) {
            let empty_threshold = 0.05 * size * size;
            if packer.empty_area() < empty_threshold || high < 1.05 * low {
                return size;
            } else {
                high = size;
            }
        } else {
            low = size;
        }
    }
}

fn get_layout(
    padding_ratio: f64,
    font: &Font<'static>,
    chars: &[char],
    size: f64,
) -> Option<PackerNode<char>> {
    let padding = padding_ratio * 2.0;
    let mut packer = PackerNode::new(size, size);
    let scale = rusttype::Scale::uniform(1.0);
    let mut array: Vec<_> = chars.iter()
        .map(|ch| {
            let glyph = font.glyph(*ch).scaled(scale);
            let rect = glyph.exact_bounding_box().expect("No bounding box?");
            let width = rect.width() as f64 + padding;
            let height = rect.height() as f64 + padding;
            (*ch, width, height)
        })
        .collect();
    array.sort_unstable_by(|a, b| {
        (a.1 * a.2).partial_cmp(&(b.1 * b.2)).unwrap().reverse()
    });
    for (ch, width, height) in array.into_iter() {
        if packer.add(ch, width, height).is_err() {
            return None;
        }
    }
    Some(packer)
}
