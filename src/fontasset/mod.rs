use std::path::{Path, PathBuf};
use std::collections::HashMap;

use rusttype::Font;

use crate::progressbar::ProgressBar;

mod render_char;
mod output;
mod settings;

use render_char::render_char;
pub use settings::{AssetSize, Settings, FontFamily};

trait GlyphSizeExt {
    fn glyph_size(&self) -> (f32, f32);
}

impl<'a> GlyphSizeExt for rusttype::ScaledGlyph<'a> {
    fn glyph_size(&self) -> (f32, f32) {
        if let Some(rect) = self.exact_bounding_box() {
            (rect.width(), rect.height())
        } else {
            (0.0, 0.0)
        }
    }
}

pub fn build_fontasset<P: AsRef<Path>>(
    ttf_family: FontFamily<'_, '_, '_, '_>,
    dest_file: P,
    settings: Settings,
) {

    let fonts = ttf_family.parse();

    let fonts = (
        fonts.normal.unwrap(),
        fonts.bold.map(Result::unwrap),
        fonts.italic.map(Result::unwrap),
        fonts.bold_italic.map(Result::unwrap),
    );

    let get_size = |font: &Font<'_>| {
        get_packed_size(settings.padding_ratio, font, &settings.chars)
    };

    let sizes = (
        get_size(&fonts.0),
        fonts.1.as_ref().map_or(0.0, get_size),
        fonts.2.as_ref().map_or(0.0, get_size),
        fonts.3.as_ref().map_or(0.0, get_size),
    );

    let max = sizes.0.max(sizes.1).max(sizes.2).max(sizes.3);

    let layout = |font: &Font<'_>| {
        let packer = get_layout(
            settings.padding_ratio,
            font,
            &settings.chars,
            max,
        ).expect("Failed to pack a font that previously fit in a larger box");
        packer.positions
    };

    let uniform = (
        layout(&fonts.0),
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
        1,
        fonts.1.is_some() as usize,
        fonts.2.is_some() as usize,
        fonts.3.is_some() as usize,
    );
    let bold_channel = fonts.1.as_ref()
        .map(|_| 1);
    let italic_channel = fonts.2.as_ref()
        .map(|_| bold_channel.unwrap_or(0) + 1);
    let bold_italic_channel = fonts.3.as_ref()
        .map(|_| italic_channel.or(bold_channel).unwrap_or(0) + 1);
    let channels = channels.0 + channels.1 + channels.2 + channels.3;
    let channels = if channels == 2 { 3 } else { channels };

    let mut progressbar = if settings.progressbar {
        ProgressBar::best("Packing Glyphs")
    } else {
        ProgressBar::none()
    };
    let mut output = settings.chars.iter()
        .enumerate()
        .fold(
            output::FontOutput::new(
                texture_size,
                texture_size,
                channels,
                font_size,
                settings.padding_ratio,
                bold_channel,
                italic_channel,
                bold_italic_channel,
            ),
            |mut buffer_data, (chindex, ch)| {
                let mut channel = 0;
                let tex_size = texture_size;
                let nchans = channels;
                let ch_shared = render_char::CharToRender {
                    ch: *ch,
                    dest_scale,
                    ref_scale,
                    x: 0.0,
                    y: 0.0,
                    chan: 0,
                    padding,
                    norm_padding: settings.padding_ratio as f32,
                };
                let mut dest_buffer = render_char::DestBuffer {
                    buffer: &mut buffer_data,
                    tex_size,
                    nchans,
                };
                let (x, y) = uniform.0[&ch];
                render_char(
                    &fonts.0,
                    render_char::CharToRender {
                        x, y, chan: channel, ..ch_shared
                    },
                    &mut dest_buffer,
                );
                channel += 1;
                if let Some(font) = &fonts.1 {
                    let (x, y) = uniform.1.as_ref().unwrap()[&ch];
                    render_char(
                        font,
                        render_char::CharToRender {
                            x, y, chan: channel, ..ch_shared
                        },
                        &mut dest_buffer,
                    );
                    channel += 1;
                }
                if let Some(font) = &fonts.2 {
                    let (x, y) = uniform.2.as_ref().unwrap()[&ch];
                    render_char(
                        font,
                        render_char::CharToRender {
                            x, y, chan: channel, ..ch_shared
                        },
                        &mut dest_buffer,
                    );
                    channel += 1;
                }
                if let Some(font) = &fonts.3 {
                    let (x, y) = uniform.3.as_ref().unwrap()[&ch];
                    render_char(
                        font,
                        render_char::CharToRender {
                            x, y, chan: channel, ..ch_shared
                        },
                        &mut dest_buffer,
                    );
                    channel += 1;
                }
                progressbar.update(chindex, settings.chars.len());
                buffer_data
            }
        );
    std::mem::drop(progressbar);
    let write_failure = "Failed to write to output file";
    output.write(dest_file).expect(write_failure);
}

fn get_packed_size(padding_ratio: f64, font: &Font, chars: &[char])
    -> f64
{
    let scale = rusttype::Scale::uniform(1.0);
    let glyphs = font.glyphs_for(chars.iter().copied())
        .map(|glyph| glyph.scaled(scale));

    let padding = padding_ratio * 2.0;
    let mut sum_width = 0.0;
    let mut sum_height = 0.0;
    for glyph in glyphs {
        let (width, height) = glyph.glyph_size();
        sum_width += width + padding as f32;
        sum_height += height + padding as f32;
    }

    let mut high: f64 = sum_width.max(sum_height).into();
    high *= 1.1;
    let mut low = 0.0f64;

    loop {
        let size = (high + low) / 2.0;
        if let Some(layout) = get_layout(
            padding_ratio,
            font,
            chars,
            size,
        ) {
            let empty_threshold = 0.05 * size * size;
            if layout.empty_area < empty_threshold || high < 1.05 * low {
                return size;
            } else {
                high = size;
            }
        } else {
            low = size;
        }
    }
}

struct LayoutResult {
    positions: HashMap<char, (f64, f64)>,
    empty_area: f64,
}

fn get_layout(
    padding_ratio: f64,
    font: &Font,
    chars: &[char],
    size: f64,
) -> Option<LayoutResult> {
    let padding = (padding_ratio * 2.0 * 100_000.0) as i32;
    let config = rect_packer::Config {
        width: (size * 100_000.0).floor() as i32,
        height: (size * 100_000.0).floor() as i32,
        border_padding: 0,
        rectangle_padding: 0,
    };
    let mut packer = rect_packer::Packer::new(config);
    let scale = rusttype::Scale::uniform(1.0);
    let mut array: Vec<_> = chars.iter()
        .map(|ch| {
            let glyph = font.glyph(*ch).scaled(scale);
            let (width, height) = glyph.glyph_size();
            let width = (width * 100_000.0).ceil() as i32 + padding;
            let height = (height * 100_000.0).ceil() as i32 + padding;
            (*ch, width, height)
        })
        .collect();
    /* rect_packer doesn't seem to indicate it needs this...
    array.sort_unstable_by(|a, b| {
        (a.1 * a.2).partial_cmp(&(b.1 * b.2)).unwrap().reverse()
    });
    */
    let mut positions = HashMap::new();
    let mut empty_area = size * size;
    for (ch, width, height) in array.into_iter() {
        let rect = packer.pack(width, height, false)?;
        let x = rect.x as f64 / 100_000.0;
        let y = rect.y as f64 / 100_000.0;
        positions.insert(ch, (x / size, y / size));
        let width = width as f64 / 100_000.0;
        let height = height as f64 / 100_000.0;
        empty_area -= width * height;
    }
    Some(LayoutResult { positions, empty_area })
}
