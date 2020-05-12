use std::path::PathBuf;

use sdl2::ttf::Font;
use sdl2::surface::Surface;
use sdl2::pixels::{Color, Palette, PixelFormatEnum};

pub struct RenderResult {
    pub character: char,
    pub width: u32,
    //pub first_data_row: u32,
    //pub last_data_row: u32,
}

pub fn render_glyph(
    ch: char,
    font: &Font<'_, '_>,
    target_size: u16,
    max_distance: u16,
    output_filename: PathBuf,
) -> Result<RenderResult, String> {
    if output_filename.exists() {
        println!("use existing: {:?}", output_filename);
        let existing = Surface::load_bmp(output_filename)?;
        return Ok(RenderResult {
            character: ch,
            width: existing.width(),
        });
    }
    let w = std::u8::MAX;
    let glyph = font.render_char(ch)
        .solid(Color::RGB(w,w,w))
        .map_err(|err| format!("{}", err))?;
    let padding = (max_distance as u32);
    let height = target_size.into();
    let (glyph_width, glyph_height) = (glyph.width(), glyph.height());
    let glyph_pitch = glyph.pitch() as usize;
    let width = glyph_width * height / glyph_height;
    let shades_of_gray = (0..=std::u8::MAX)
        .map(|v| Color::RGB(v,v,v))
        .collect::<Vec<_>>();
    let palette = Palette::with_colors(&shades_of_gray)?;
    let full_width = width + 2 * padding;
    let full_height = height + 2 * padding;
    let mut sdf = Surface::new(
        full_width,
        full_height,
        PixelFormatEnum::Index8,
    )?;
    sdf.set_palette(&palette)?;
    let pitch = sdf.pitch() as usize;
    let glyph_width_f = glyph_width as f64;
    let glyph_height_f = glyph_height as f64;
    let to_glyph_single = |val: usize, len: u32, glyph_len| {
        let val = (val as f64) - (padding as f64);
        let val = val / (len as f64);
        val * glyph_len
    };
    let mut first_data_row = full_height;
    let mut last_data_row = 0;
    glyph.with_lock(|source| {
        let state_at = |x: f64, y: f64| -> bool {
            let oob = (
                x < 0.0
                || x >= glyph_width_f
                || y < 0.0
                || y >= glyph_height_f
            );
            if oob {
                false
            } else {
                let src_x = x.round() as usize;
                let src_y = y.round() as usize;
                source[src_y * glyph_pitch + src_x] != 0
            }
        };
        sdf.with_lock_mut(|dest| {
            for y in 0..(full_height as usize) {
                let glyph_y = (to_glyph_single)(y, height, glyph_height_f);
                let mut empty_row = true;
                for x in 0..(full_width as usize) {
                    let dest_px = &mut dest[y * pitch + x];
                    let glyph_x = (to_glyph_single)(x, width, glyph_width_f);
                    let state = (state_at)(glyph_x, glyph_y);
                    let min_dist = 
                        (0..glyph_height).into_iter().flat_map(|gy| {
                            (0..glyph_width).into_iter().filter_map(move |gx| {
                                let gx = gx as f64;
                                let gy = gy as f64;
                                if state == (state_at)(gx, gy) {
                                    None
                                } else {
                                    let a2 = (glyph_x - gx).powi(2);
                                    let b2 = (glyph_y - gy).powi(2);
                                    Some((a2 + b2).sqrt())
                                }
                            })
                        })
                        .min_by(|a, b| {
                            a.partial_cmp(b)
                            .expect("Somehow got a NaN distance")
                        })
                        .expect("Somehow got no distances");
                    let min_dist = min_dist * (width as f64) / (glyph_width as f64);
                    let min_dist = if state { min_dist } else { -min_dist };
                    let frac_dist = min_dist / (max_distance as f64);
                    let frac_dist = (frac_dist + 1.0) / 2.0;
                    let frac_dist = frac_dist.max(0.0).min(1.0);
                    *dest_px = (frac_dist * 255.0).floor() as u8;
                    if *dest_px > 0 {
                        empty_row = false;
                    }
                }
                if !empty_row {
                    last_data_row = y as u32;
                    first_data_row = first_data_row.min(y as u32);
                }
            }
        });
    });
    sdf.save_bmp(output_filename);
    Ok(RenderResult {
        character: ch,
        width: full_width,
        //first_data_row,
        //last_data_row,
    })
}
