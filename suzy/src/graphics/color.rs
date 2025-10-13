/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::fmt::{self, LowerHex, UpperHex};

use crate::{
    animation::{Lerp, LerpDistance},
    units::QuantizeU8,
};

/// A type which represents a color with 32 bit floating point components.
///
/// Colors can be crated a number of ways:
///
/// ```rust
/// # use suzy::graphics::Color;
/// let color0 = Color::from_rgba(0.1980693, 0.13843162, 0.8549927, 1.0);
/// let color1 = Color::from_rgba8(123, 104, 238, 255);
/// let color2: Color = "#7B68EE".parse().unwrap();
/// assert_eq!(color0, color1);
/// assert_eq!(color1, color2);
/// ```
///
/// Colors can be formatted with hex style to get familar results:
/// ```rust
/// # use suzy::graphics::Color;
/// let color = Color::from_rgba8(70, 130, 180, 255);
/// let string = format!("{:X}", color);
/// assert_eq!(string, "#4682B4FF");
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color with the specified RGBA components, where 1.0
    /// represents the maximum for that component.
    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    /// Create a new color with the specified RGBA components, where 255
    /// represents the maximum for that component.
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color {
            r: srgb_decompress(r),
            g: srgb_decompress(g),
            b: srgb_decompress(b),
            a: f32::from(a) / 255.0,
        }
    }

    /// Get the RGBA components of this color, as integers, where 255
    /// represents the maximum for that component.
    pub fn rgba8(&self) -> [u8; 4] {
        [
            srgb_compress(self.r),
            srgb_compress(self.g),
            srgb_compress(self.b),
            self.a.quantize_u8(),
        ]
    }
}

impl std::ops::Mul for Color {
    type Output = Color;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
            a: self.a * rhs.a,
        }
    }
}

impl Lerp for Color {
    type Output = Color;
    fn lerp(from: &Self, to: &Self, t: f32) -> Self::Output {
        Color {
            r: Lerp::lerp(&from.r, &to.r, t),
            g: Lerp::lerp(&from.g, &to.g, t),
            b: Lerp::lerp(&from.b, &to.b, t),
            a: Lerp::lerp(&from.a, &to.a, t),
        }
    }
}

impl LerpDistance for Color {
    fn lerp_distance(a: &Self, b: &Self) -> f32 {
        f32::sqrt(
            (a.r - b.r).powi(2)
                + (a.g - b.g).powi(2)
                + (a.b - b.b).powi(2)
                + (a.a - b.a).powi(2),
        )
    }
}

/// An error returned from a failed attempt to parse a string as a color.
#[derive(Copy, Clone, Debug, Default)]
pub struct ParseColorError;

impl From<std::num::ParseIntError> for ParseColorError {
    fn from(_orig: std::num::ParseIntError) -> Self {
        Self
    }
}

impl std::str::FromStr for Color {
    type Err = ParseColorError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(hex_part) = s.strip_prefix('#') {
            if hex_part.len() == 6 || hex_part.len() == 8 {
                let mut int = u32::from_str_radix(hex_part, 16)?;
                if hex_part.len() == 6 {
                    int = (int << 8_u8) | 0xFF;
                }
                let bytes = int.to_be_bytes();
                Ok(Self::from_rgba8(bytes[0], bytes[1], bytes[2], bytes[3]))
            } else {
                Err(ParseColorError {})
            }
        } else {
            Self::name_to_color(s).ok_or(ParseColorError {})
        }
    }
}

impl LowerHex for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:08x}", u32::from_be_bytes(self.rgba8()))
    }
}

impl UpperHex for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:08X}", u32::from_be_bytes(self.rgba8()))
    }
}

const SRGB_A: f32 = 0.055;
const SRGB_1A: f32 = 1.0 + SRGB_A;
const SRGB_GAMMA: f32 = 2.4;
const SRGB_PHI: f32 = 12.92;
/// The compressed transition point would be `SRGB_A / (SRGB_GAMMA - 1.0)` but
/// because PHI is rounded above, it's adjusted to this.
const SRGB_TRANS_COMPRESSED: f32 = 0.04045;
const SRGB_TRANS_UNCOMPRESSED: f32 = SRGB_TRANS_COMPRESSED / SRGB_PHI;
const SRGB_TRANS_COMPRESSED_U8: u8 = {
    match SRGB_TRANS_COMPRESSED * 255.0 {
        10.0..=10.9 => 10,
        _ => panic!(),
    }
};

fn srgb_decompress(value: u8) -> f32 {
    let vf = f32::from(value) / 255.0;
    if value > SRGB_TRANS_COMPRESSED_U8 {
        ((vf + SRGB_A) / SRGB_1A).powf(SRGB_GAMMA)
    } else {
        vf / SRGB_PHI
    }
}

fn srgb_compress(value: f32) -> u8 {
    let curved = if value > SRGB_TRANS_UNCOMPRESSED {
        value.powf(1.0 / SRGB_GAMMA) * SRGB_1A - SRGB_A
    } else {
        value * SRGB_PHI
    };
    curved.quantize_u8()
}

macro_rules! cc {
    ($($name:ident = $code:literal,)*) => {
        $(
            /// A named color constant
            pub const $name: Color = {
                let rgb: u32 = $code;
                let r = (rgb >> 16_u8) as u8;
                let g = ((rgb >> 8_u8) & 0xFF) as u8;
                let b = (rgb & 0xFF) as u8;
                let r = (r as f32) / 255.0;
                let g = (g as f32) / 255.0;
                let b = (b as f32) / 255.0;
                Color {
                    r, g, b, a: 1.0,
                }
            };
        )*
    };
}

impl Color {
    cc! {
        ALICE_BLUE = 0xF0F8FF,
        ANTIQUE_WHITE = 0xFAEBD7,
        AQUA = 0x00FFFF,
        AQUAMARINE = 0x7FFFD4,
        AZURE = 0xF0FFFF,
        BEIGE = 0xF5F5DC,
        BISQUE = 0xFFE4C4,
        BLACK = 0x000000,
        BLANCHED_ALMOND = 0xFFEBCD,
        BLUE = 0x0000FF,
        BLUE_VIOLET = 0x8A2BE2,
        BROWN = 0xA52A2A,
        BURLY_WOOD = 0xDEB887,
        CADET_BLUE = 0x5F9EA0,
        CHARTREUSE = 0x7FFF00,
        CHOCOLATE = 0xD2691E,
        CORAL = 0xFF7F50,
        CORNFLOWER_BLUE = 0x6495ED,
        CORNSILK = 0xFFF8DC,
        CRIMSON = 0xDC143C,
        CYAN = 0x00FFFF,
        DARK_BLUE = 0x00008B,
        DARK_CYAN = 0x008B8B,
        DARK_GOLDEN_ROD = 0xB8860B,
        DARK_GRAY = 0xA9A9A9,
        DARK_GREY = 0xA9A9A9,
        DARK_GREEN = 0x006400,
        DARK_KHAKI = 0xBDB76B,
        DARK_MAGENTA = 0x8B008B,
        DARK_OLIVE_GREEN = 0x556B2F,
        DARK_ORANGE = 0xFF8C00,
        DARK_ORCHID = 0x9932CC,
        DARK_RED = 0x8B0000,
        DARK_SALMON = 0xE9967A,
        DARK_SEA_GREEN = 0x8FBC8F,
        DARK_SLATE_BLUE = 0x483D8B,
        DARK_SLATE_GRAY = 0x2F4F4F,
        DARK_SLATE_GREY = 0x2F4F4F,
        DARK_TURQUOISE = 0x00CED1,
        DARK_VIOLET = 0x9400D3,
        DEEP_PINK = 0xFF1493,
        DEEP_SKY_BLUE = 0x00BFFF,
        DIM_GRAY = 0x696969,
        DIM_GREY = 0x696969,
        DODGER_BLUE = 0x1E90FF,
        FIRE_BRICK = 0xB22222,
        FLORAL_WHITE = 0xFFFAF0,
        FOREST_GREEN = 0x228B22,
        FUCHSIA = 0xFF00FF,
        GAINSBORO = 0xDCDCDC,
        GHOST_WHITE = 0xF8F8FF,
        GOLD = 0xFFD700,
        GOLDEN_ROD = 0xDAA520,
        GRAY = 0x808080,
        GREY = 0x808080,
        GREEN = 0x008000,
        GREEN_YELLOW = 0xADFF2F,
        HONEY_DEW = 0xF0FFF0,
        HOT_PINK = 0xFF69B4,
        INDIAN_RED = 0xCD5C5C,
        INDIGO = 0x4B0082,
        IVORY = 0xFFFFF0,
        KHAKI = 0xF0E68C,
        LAVENDER = 0xE6E6FA,
        LAVENDER_BLUSH = 0xFFF0F5,
        LAWN_GREEN = 0x7CFC00,
        LEMON_CHIFFON = 0xFFFACD,
        LIGHT_BLUE = 0xADD8E6,
        LIGHT_CORAL = 0xF08080,
        LIGHT_CYAN = 0xE0FFFF,
        LIGHT_GOLDEN_ROD_YELLOW = 0xFAFAD2,
        LIGHT_GRAY = 0xD3D3D3,
        LIGHT_GREY = 0xD3D3D3,
        LIGHT_GREEN = 0x90EE90,
        LIGHT_PINK = 0xFFB6C1,
        LIGHT_SALMON = 0xFFA07A,
        LIGHT_SEA_GREEN = 0x20B2AA,
        LIGHT_SKY_BLUE = 0x87CEFA,
        LIGHT_SLATE_GRAY = 0x778899,
        LIGHT_SLATE_GREY = 0x778899,
        LIGHT_STEEL_BLUE = 0xB0C4DE,
        LIGHT_YELLOW = 0xFFFFE0,
        LIME = 0x00FF00,
        LIME_GREEN = 0x32CD32,
        LINEN = 0xFAF0E6,
        MAGENTA = 0xFF00FF,
        MAROON = 0x800000,
        MEDIUM_AQUA_MARINE = 0x66CDAA,
        MEDIUM_BLUE = 0x0000CD,
        MEDIUM_ORCHID = 0xBA55D3,
        MEDIUM_PURPLE = 0x9370DB,
        MEDIUM_SEA_GREEN = 0x3CB371,
        MEDIUM_SLATE_BLUE = 0x7B68EE,
        MEDIUM_SPRING_GREEN = 0x00FA9A,
        MEDIUM_TURQUOISE = 0x48D1CC,
        MEDIUM_VIOLET_RED = 0xC71585,
        MIDNIGHT_BLUE = 0x191970,
        MINT_CREAM = 0xF5FFFA,
        MISTY_ROSE = 0xFFE4E1,
        MOCCASIN = 0xFFE4B5,
        NAVAJO_WHITE = 0xFFDEAD,
        NAVY = 0x000080,
        OLD_LACE = 0xFDF5E6,
        OLIVE = 0x808000,
        OLIVE_DRAB = 0x6B8E23,
        ORANGE = 0xFFA500,
        ORANGE_RED = 0xFF4500,
        ORCHID = 0xDA70D6,
        PALE_GOLDEN_ROD = 0xEEE8AA,
        PALE_GREEN = 0x98FB98,
        PALE_TURQUOISE = 0xAFEEEE,
        PALE_VIOLET_RED = 0xDB7093,
        PAPAYA_WHIP = 0xFFEFD5,
        PEACH_PUFF = 0xFFDAB9,
        PERU = 0xCD853F,
        PINK = 0xFFC0CB,
        PLUM = 0xDDA0DD,
        POWDER_BLUE = 0xB0E0E6,
        PURPLE = 0x800080,
        REBECCA_PURPLE = 0x663399,
        RED = 0xFF0000,
        ROSY_BROWN = 0xBC8F8F,
        ROYAL_BLUE = 0x4169E1,
        SADDLE_BROWN = 0x8B4513,
        SALMON = 0xFA8072,
        SANDY_BROWN = 0xF4A460,
        SEA_GREEN = 0x2E8B57,
        SEA_SHELL = 0xFFF5EE,
        SIENNA = 0xA0522D,
        SILVER = 0xC0C0C0,
        SKY_BLUE = 0x87CEEB,
        SLATE_BLUE = 0x6A5ACD,
        SLATE_GRAY = 0x708090,
        SLATE_GREY = 0x708090,
        SNOW = 0xFFFAFA,
        SPRING_GREEN = 0x00FF7F,
        STEEL_BLUE = 0x4682B4,
        TAN = 0xD2B48C,
        TEAL = 0x008080,
        THISTLE = 0xD8BFD8,
        TOMATO = 0xFF6347,
        TURQUOISE = 0x40E0D0,
        VIOLET = 0xEE82EE,
        WHEAT = 0xF5DEB3,
        WHITE = 0xFFFFFF,
        WHITE_SMOKE = 0xF5F5F5,
        YELLOW = 0xFFFF00,
        YELLOW_GREEN = 0x9ACD32,
    }

    pub(crate) fn name_to_color(name: &str) -> Option<Color> {
        Some(match name {
            "aliceblue" => Self::ALICE_BLUE,
            "antiquewhite" => Self::ANTIQUE_WHITE,
            "aqua" => Self::AQUA,
            "aquamarine" => Self::AQUAMARINE,
            "azure" => Self::AZURE,
            "beige" => Self::BEIGE,
            "bisque" => Self::BISQUE,
            "black" => Self::BLACK,
            "blanchedalmond" => Self::BLANCHED_ALMOND,
            "blue" => Self::BLUE,
            "blueviolet" => Self::BLUE_VIOLET,
            "brown" => Self::BROWN,
            "burlywood" => Self::BURLY_WOOD,
            "cadetblue" => Self::CADET_BLUE,
            "chartreuse" => Self::CHARTREUSE,
            "chocolate" => Self::CHOCOLATE,
            "coral" => Self::CORAL,
            "cornflowerblue" => Self::CORNFLOWER_BLUE,
            "cornsilk" => Self::CORNSILK,
            "crimson" => Self::CRIMSON,
            "cyan" => Self::CYAN,
            "darkblue" => Self::DARK_BLUE,
            "darkcyan" => Self::DARK_CYAN,
            "darkgoldenrod" => Self::DARK_GOLDEN_ROD,
            "darkgray" => Self::DARK_GRAY,
            "darkgrey" => Self::DARK_GREY,
            "darkgreen" => Self::DARK_GREEN,
            "darkkhaki" => Self::DARK_KHAKI,
            "darkmagenta" => Self::DARK_MAGENTA,
            "darkolivegreen" => Self::DARK_OLIVE_GREEN,
            "darkorange" => Self::DARK_ORANGE,
            "darkorchid" => Self::DARK_ORCHID,
            "darkred" => Self::DARK_RED,
            "darksalmon" => Self::DARK_SALMON,
            "darkseagreen" => Self::DARK_SEA_GREEN,
            "darkslateblue" => Self::DARK_SLATE_BLUE,
            "darkslategray" => Self::DARK_SLATE_GRAY,
            "darkslategrey" => Self::DARK_SLATE_GREY,
            "darkturquoise" => Self::DARK_TURQUOISE,
            "darkviolet" => Self::DARK_VIOLET,
            "deeppink" => Self::DEEP_PINK,
            "deepskyblue" => Self::DEEP_SKY_BLUE,
            "dimgray" => Self::DIM_GRAY,
            "dimgrey" => Self::DIM_GREY,
            "dodgerblue" => Self::DODGER_BLUE,
            "firebrick" => Self::FIRE_BRICK,
            "floralwhite" => Self::FLORAL_WHITE,
            "forestgreen" => Self::FOREST_GREEN,
            "fuchsia" => Self::FUCHSIA,
            "gainsboro" => Self::GAINSBORO,
            "ghostwhite" => Self::GHOST_WHITE,
            "gold" => Self::GOLD,
            "goldenrod" => Self::GOLDEN_ROD,
            "gray" => Self::GRAY,
            "grey" => Self::GREY,
            "green" => Self::GREEN,
            "greenyellow" => Self::GREEN_YELLOW,
            "honeydew" => Self::HONEY_DEW,
            "hotpink" => Self::HOT_PINK,
            "indianred" => Self::INDIAN_RED,
            "indigo" => Self::INDIGO,
            "ivory" => Self::IVORY,
            "khaki" => Self::KHAKI,
            "lavender" => Self::LAVENDER,
            "lavenderblush" => Self::LAVENDER_BLUSH,
            "lawngreen" => Self::LAWN_GREEN,
            "lemonchiffon" => Self::LEMON_CHIFFON,
            "lightblue" => Self::LIGHT_BLUE,
            "lightcoral" => Self::LIGHT_CORAL,
            "lightcyan" => Self::LIGHT_CYAN,
            "lightgoldenrodyellow" => Self::LIGHT_GOLDEN_ROD_YELLOW,
            "lightgray" => Self::LIGHT_GRAY,
            "lightgrey" => Self::LIGHT_GREY,
            "lightgreen" => Self::LIGHT_GREEN,
            "lightpink" => Self::LIGHT_PINK,
            "lightsalmon" => Self::LIGHT_SALMON,
            "lightseagreen" => Self::LIGHT_SEA_GREEN,
            "lightskyblue" => Self::LIGHT_SKY_BLUE,
            "lightslategray" => Self::LIGHT_SLATE_GRAY,
            "lightslategrey" => Self::LIGHT_SLATE_GREY,
            "lightsteelblue" => Self::LIGHT_STEEL_BLUE,
            "lightyellow" => Self::LIGHT_YELLOW,
            "lime" => Self::LIME,
            "limegreen" => Self::LIME_GREEN,
            "linen" => Self::LINEN,
            "magenta" => Self::MAGENTA,
            "maroon" => Self::MAROON,
            "mediumaquamarine" => Self::MEDIUM_AQUA_MARINE,
            "mediumblue" => Self::MEDIUM_BLUE,
            "mediumorchid" => Self::MEDIUM_ORCHID,
            "mediumpurple" => Self::MEDIUM_PURPLE,
            "mediumseagreen" => Self::MEDIUM_SEA_GREEN,
            "mediumslateblue" => Self::MEDIUM_SLATE_BLUE,
            "mediumspringgreen" => Self::MEDIUM_SPRING_GREEN,
            "mediumturquoise" => Self::MEDIUM_TURQUOISE,
            "mediumvioletred" => Self::MEDIUM_VIOLET_RED,
            "midnightblue" => Self::MIDNIGHT_BLUE,
            "mintcream" => Self::MINT_CREAM,
            "mistyrose" => Self::MISTY_ROSE,
            "moccasin" => Self::MOCCASIN,
            "navajowhite" => Self::NAVAJO_WHITE,
            "navy" => Self::NAVY,
            "oldlace" => Self::OLD_LACE,
            "olive" => Self::OLIVE,
            "olivedrab" => Self::OLIVE_DRAB,
            "orange" => Self::ORANGE,
            "orangered" => Self::ORANGE_RED,
            "orchid" => Self::ORCHID,
            "palegoldenrod" => Self::PALE_GOLDEN_ROD,
            "palegreen" => Self::PALE_GREEN,
            "paleturquoise" => Self::PALE_TURQUOISE,
            "palevioletred" => Self::PALE_VIOLET_RED,
            "papayawhip" => Self::PAPAYA_WHIP,
            "peachpuff" => Self::PEACH_PUFF,
            "peru" => Self::PERU,
            "pink" => Self::PINK,
            "plum" => Self::PLUM,
            "powderblue" => Self::POWDER_BLUE,
            "purple" => Self::PURPLE,
            "rebeccapurple" => Self::REBECCA_PURPLE,
            "red" => Self::RED,
            "rosybrown" => Self::ROSY_BROWN,
            "royalblue" => Self::ROYAL_BLUE,
            "saddlebrown" => Self::SADDLE_BROWN,
            "salmon" => Self::SALMON,
            "sandybrown" => Self::SANDY_BROWN,
            "seagreen" => Self::SEA_GREEN,
            "seashell" => Self::SEA_SHELL,
            "sienna" => Self::SIENNA,
            "silver" => Self::SILVER,
            "skyblue" => Self::SKY_BLUE,
            "slateblue" => Self::SLATE_BLUE,
            "slategray" => Self::SLATE_GRAY,
            "slategrey" => Self::SLATE_GREY,
            "snow" => Self::SNOW,
            "springgreen" => Self::SPRING_GREEN,
            "steelblue" => Self::STEEL_BLUE,
            "tan" => Self::TAN,
            "teal" => Self::TEAL,
            "thistle" => Self::THISTLE,
            "tomato" => Self::TOMATO,
            "turquoise" => Self::TURQUOISE,
            "violet" => Self::VIOLET,
            "wheat" => Self::WHEAT,
            "white" => Self::WHITE,
            "whitesmoke" => Self::WHITE_SMOKE,
            "yellow" => Self::YELLOW,
            "yellowgreen" => Self::YELLOW_GREEN,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srgb_decompress_compress_lossless() {
        for x in 0..=u8::MAX {
            assert_eq!(x, srgb_compress(srgb_decompress(x)));
        }
    }
}
