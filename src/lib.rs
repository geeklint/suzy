use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::io::Write;

use sha2::Digest;
use rayon::prelude::*;

mod render_glyph;

thread_local! {
    static CACHE_PATH: RefCell<Vec<PathBuf>> = RefCell::new(
        vec![".cache/fontasset_builder".into()]
    );
}

pub struct CachePathContext(());

impl CachePathContext {
    fn new<P: Into<PathBuf>>(path: P) -> Self {
        CACHE_PATH.with(|cell| cell.borrow_mut().push(path.into()));
        CachePathContext(())
    }
}

impl Drop for CachePathContext {
    fn drop(&mut self) {
        CACHE_PATH.with(|cell| cell.borrow_mut().pop());
    }
}

pub fn use_cache_path<P: Into<PathBuf>>(path: P) -> CachePathContext {
    CachePathContext::new(path)
}

pub struct Settings<C>
    where C: IntoParallelIterator<Item=char>
{
    pub chars: C,
    pub target_size: u16,
    pub max_distance: u16,
}

pub fn build_fontasset<S, D, C>(
    source_ttf: S,
    dest_file: D,
    settings: Settings<C>,
)
    where
        S: AsRef<Path>,
        D: AsRef<Path>,
        C: IntoParallelIterator<Item=char>,
{
    let mut cache_path = CACHE_PATH.with(|cell| {
        cell.borrow().last().expect("CACHE_PATH is empty?").clone()
    });
    println!("loading font file...");
    let font_bytes = std::fs::read(source_ttf)
        .expect("Failed to read font file");
    let hash = sha2::Sha224::digest(&font_bytes);
    cache_path.push(hex::encode(hash));
    std::fs::create_dir_all(&cache_path);

    let sdl = sdl2::init().expect("SDL failed to init");
    let ttf = sdl2::ttf::init().expect("SDL_ttf failed to init");

    let target_size = settings.target_size;
    let large_size = std::cmp::max(target_size, 512);
    let get_font = || {
        let ops = sdl2::rwops::RWops::from_bytes(&font_bytes)?;
        ttf.load_font_from_rwops(ops, large_size)
    };
    let max_distance = settings.max_distance;
    
    let mut result = settings.chars.into_par_iter()
        .map_init(get_font, {
            let cache_path = cache_path.clone();
            move |maybe_font, ch| {
                match maybe_font {
                    Ok(font) => {
                        let mut output_filename = cache_path.clone();
                        output_filename.push(format!("{:x}.bmp", ch as u32));
                        match render_glyph::render_glyph(
                            ch,
                            &font,
                            target_size,
                            max_distance,
                            output_filename,
                        ) {
                            Ok(thing) => Ok(thing),
                            Err(err) => Err(format!(
                                "glyph '{}' ({:x}) failed to render: {}",
                                ch,
                                ch as u32,
                                err,
                            )),
                        }
                    },
                    Err(err) => Err(
                        format!("Font failed to parse: {}", err)
                    ),
                }
            }
        })
        .filter_map(|result| {
            match result {
                Ok(thing) => Some(thing),
                Err(err) => {
                    eprintln!("{}", err);
                    None
                }
            }
        })
        .collect::<Vec<_>>();
    println!("stitching atlas...");
    result.sort_unstable_by_key(|r| r.width);
    let row_height = (target_size as usize) + 2 * (max_distance as usize);
    let mut rows: Vec<Vec<&render_glyph::RenderResult>> = vec![vec![]];
    let mut current = result.split_last();
    let mut output_height = (rows.len() * row_height).next_power_of_two();
    let row_size = |row: & &mut Vec<&render_glyph::RenderResult>| -> usize {
        row.iter().map(|c| c.width as usize).sum()
    };
    while let Some((element, remaining)) = current {
        let shortest_row = rows.iter_mut()
            .min_by_key(row_size)
            .unwrap();
        shortest_row.push(element);
        if (row_size)(&shortest_row) > output_height {
            for row in rows.iter_mut() {
                row.clear();
            }
            output_height = (rows.len() * row_height).next_power_of_two();
            rows.push(Vec::new());
            current = result.split_last();
        } else {
            current = remaining.split_last();
        }
    }
    use sdl2::surface::Surface;
    use sdl2::pixels::{Color, Palette, PixelFormatEnum};
    use sdl2::rect::Rect;
    let shades_of_gray = (0..=std::u8::MAX)
        .map(|v| Color::RGB(v,v,v))
        .collect::<Vec<_>>();
    let palette = Palette::with_colors(&shades_of_gray).unwrap();
    let mut atlas = Surface::new(
        output_height as u32,
        output_height as u32,
        PixelFormatEnum::Index8,
    ).unwrap();
    atlas.set_palette(&palette).unwrap();
    let dim = output_height as f64;
    let height_coord = row_height as f64 / dim;
    let mut coords = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let y = (i * row_height) as i32;
        // opengl coords go from bottom
        let y_coord = y as f64 / dim;
        let mut x = 0;
        for ch_result in row.iter() {
            let mut filename = cache_path.clone();
            filename.push(format!("{:x}.bmp", ch_result.character as u32));
            let source = Surface::load_bmp(filename).unwrap();
            let src_rect = Some(source.rect());
            let dst_rect = Some(
                Rect::new(x, y, source.width(), source.height())
            );
            source.blit(src_rect, &mut atlas, dst_rect).unwrap();
            coords.push((
                ch_result.character as u32,
                x as f64 / dim,
                y_coord,
                source.width() as f64 / dim,
                height_coord,
            ));
            x += source.width() as i32;
        }
    }
    atlas.save_bmp("atlas-debug.bmp");
    let write_failure = "Failed to write to output file";
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_file)
        .expect(write_failure);
    writeln!(&file, "use suzy::platform::opengl::text::FontSource;");
    coords.sort_unstable_by_key(|c| c.0);
    let coord_ref: &[(u32, f64, f64, f64, f64)] = &coords;
    atlas.with_lock(|pixels| {
        writeln!(
            &file,
            "pub const FONT: FontSource<'static, 'static> = FontSource {{
                font_size: {:.1},
                image_width: {},
                image_height: {},
                atlas_image: &{:#?},
                coords: &{:#?},
            }};",
            (target_size as f64),
            output_height,
            output_height,
            pixels,
            coord_ref,
        ).expect(write_failure);
    });
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
