use rayon::prelude::*;
use sdl2::ttf::Font;

use super::render_glyph::RenderResult;


pub fn get_ratio(
    first: char,
    second: char,
    font: &Font,
) -> Result<(char, char, f64), String> {
    let mut string = String::with_capacity(8);
    string.push(first);
    string.push(second);
    let first_width = font.size_of_char(first)
        .map_err(|err| format!("{}", err))?.0;
    let second_width = font.size_of_char(second)
        .map_err(|err| format!("{}", err))?.0;
    let unkerned_width = first_width + second_width;
    let (width, height) = font.size_of(&string)
        .map_err(|err| format!("{}", err))?;
    Ok((
        first,
        second,
        (width as f64) / (unkerned_width as f64),
    ))
}

pub fn get_pairs<'a, 'b, F, E>(
    chars: &[RenderResult],
    get_font: F,
    base_width: u32,
) -> Vec<(char, char, f64)>
    where
        F: Fn() -> Result<Font<'a, 'b>, E> + Send + Sync,
        E: std::fmt::Display
{
    let mut pairs = chars.iter()
        .flat_map(|first| {
            chars.iter().map(move |second| (first.character, second.character))
        })
        .map({ let maybe_font = (get_font)(); move |(first, second)| {
            match &maybe_font {
                Ok(font) => get_ratio(first, second, font),
                Err(err) => Err(
                    format!("Font failed to parse: {}", err)
                ),
            }
        }})
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
    pairs.sort_unstable_by_key(|v| (v.0, v.1));
    pairs
}
