use std::path::{Path, PathBuf};
use std::collections::HashMap;

use rusttype::Font;

use crate::atlas_packer::PackerNode;
use crate::progressbar::ProgressBar;

mod render_char;
mod output;
mod settings;

use render_char::render_char;
pub use settings::{AssetSize, Settings, FontFamily};

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
        let mut flat = HashMap::new();
        packer.for_each(|x, y, ch| { flat.insert(*ch, (x / max, y / max)); });
        flat
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
    let channels = channels.0 + channels.1 + channels.2 + channels.3;
    let channels = if channels == 2 { 3 } else { channels };
    assert_eq!(channels, 1);

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
    font: &Font,
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
