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

fn srgb_decompress(value: u8) -> f32 {
    let vf = f32::from(value) / 255.0;
    if value > 10 {
        ((vf + 0.055) / 1.055).powf(2.4)
    } else {
        vf / 12.92
    }
}

fn srgb_compress(value: f32) -> u8 {
    let curved = if value > 0.0031308 {
        value.powf(1.0 / 2.4) * 1.055 - 0.055
    } else {
        value * 12.92
    };
    curved.quantize_u8()
}

const fn cc(r: f32, g: f32, b: f32) -> Color {
    Color::from_rgba(r, g, b, 1.0)
}

macro_rules! doc_colors {
    ( $( $color:item )* ) => {
        $(
            /// A named color constant
            $color
        )*
    };
}

impl Color {
    doc_colors! {
    pub const ALICE_BLUE: Color = cc(0.8713671, 0.9386857, 1.0);
    pub const ANTIQUE_WHITE: Color = cc(0.9559733, 0.8307699, 0.6795425);
    pub const AQUA: Color = cc(0.0, 1.0, 1.0);
    pub const AQUAMARINE: Color = cc(0.21223076, 1.0, 0.65837485);
    pub const AZURE: Color = cc(0.8713671, 1.0, 1.0);
    pub const BEIGE: Color = cc(0.91309863, 0.91309863, 0.7156935);
    pub const BISQUE: Color = cc(1.0, 0.7758222, 0.55201143);
    pub const BLACK: Color = cc(0.0, 0.0, 0.0);
    pub const BLANCHED_ALMOND: Color = cc(1.0, 0.8307699, 0.61049557);
    pub const BLUE: Color = cc(0.0, 0.0, 1.0);
    pub const BLUE_VIOLET: Color = cc(0.2541521, 0.024157632, 0.7605245);
    pub const BROWN: Color = cc(0.37626213, 0.023153367, 0.023153367);
    pub const BURLY_WOOD: Color = cc(0.73046076, 0.47932017, 0.24228112);
    pub const CADET_BLUE: Color = cc(0.114435375, 0.34191442, 0.3515326);
    pub const CHARTREUSE: Color = cc(0.21223076, 1.0, 0.0);
    pub const CHOCOLATE: Color = cc(0.6444797, 0.14126329, 0.0129830325);
    pub const CORAL: Color = cc(1.0, 0.21223076, 0.08021982);
    pub const CORNFLOWER_BLUE: Color = cc(0.12743768, 0.30054379, 0.8468732);
    pub const CORNSILK: Color = cc(1.0, 0.9386857, 0.7156935);
    pub const CRIMSON: Color = cc(0.7156935, 0.00699541, 0.045186203);
    pub const CYAN: Color = cc(0.0, 1.0, 1.0);
    pub const DARK_BLUE: Color = cc(0.0, 0.0, 0.25818285);
    pub const DARK_CYAN: Color = cc(0.0, 0.25818285, 0.25818285);
    pub const DARK_GOLDEN_ROD: Color =
        cc(0.47932017, 0.23839757, 0.0033465358);
    pub const DARK_GRAY: Color = cc(0.39675522, 0.39675522, 0.39675522);
    pub const DARK_GREY: Color = cc(0.39675522, 0.39675522, 0.39675522);
    pub const DARK_GREEN: Color = cc(0.0, 0.12743768, 0.0);
    pub const DARK_KHAKI: Color = cc(0.50888133, 0.47353148, 0.14702727);
    pub const DARK_MAGENTA: Color = cc(0.25818285, 0.0, 0.25818285);
    pub const DARK_OLIVE_GREEN: Color = cc(0.09084171, 0.14702727, 0.02842604);
    pub const DARK_ORANGE: Color = cc(1.0, 0.26225066, 0.0);
    pub const DARK_ORCHID: Color = cc(0.31854677, 0.031896032, 0.60382736);
    pub const DARK_RED: Color = cc(0.25818285, 0.0, 0.0);
    pub const DARK_SALMON: Color = cc(0.8148466, 0.3049873, 0.19461784);
    pub const DARK_SEA_GREEN: Color = cc(0.2746773, 0.5028865, 0.2746773);
    pub const DARK_SLATE_BLUE: Color =
        cc(0.064803265, 0.046665087, 0.25818285);
    pub const DARK_SLATE_GRAY: Color = cc(0.02842604, 0.07818742, 0.07818742);
    pub const DARK_SLATE_GREY: Color = cc(0.02842604, 0.07818742, 0.07818742);
    pub const DARK_TURQUOISE: Color = cc(0.0, 0.6172066, 0.63759685);
    pub const DARK_VIOLET: Color = cc(0.29613826, 0.0, 0.65140563);
    pub const DEEP_PINK: Color = cc(1.0, 0.00699541, 0.29177064);
    pub const DEEP_SKY_BLUE: Color = cc(0.0, 0.52099556, 1.0);
    pub const DIM_GRAY: Color = cc(0.14126329, 0.14126329, 0.14126329);
    pub const DIM_GREY: Color = cc(0.14126329, 0.14126329, 0.14126329);
    pub const DODGER_BLUE: Color = cc(0.0129830325, 0.27889428, 1.0);
    pub const FIRE_BRICK: Color = cc(0.4452012, 0.015996294, 0.015996294);
    pub const FLORAL_WHITE: Color = cc(1.0, 0.9559733, 0.8713671);
    pub const FOREST_GREEN: Color = cc(0.015996294, 0.25818285, 0.015996294);
    pub const FUCHSIA: Color = cc(1.0, 0.0, 1.0);
    pub const GAINSBORO: Color = cc(0.7156935, 0.7156935, 0.7156935);
    pub const GHOST_WHITE: Color = cc(0.9386857, 0.9386857, 1.0);
    pub const GOLD: Color = cc(1.0, 0.6795425, 0.0);
    pub const GOLDEN_ROD: Color = cc(0.7011019, 0.37626213, 0.014443844);
    pub const GRAY: Color = cc(0.2158605, 0.2158605, 0.2158605);
    pub const GREY: Color = cc(0.2158605, 0.2158605, 0.2158605);
    pub const GREEN: Color = cc(0.0, 0.2158605, 0.0);
    pub const GREEN_YELLOW: Color = cc(0.41788507, 1.0, 0.02842604);
    pub const HONEY_DEW: Color = cc(0.8713671, 1.0, 0.8713671);
    pub const HOT_PINK: Color = cc(1.0, 0.14126329, 0.45641103);
    pub const INDIAN_RED: Color = cc(0.61049557, 0.107023105, 0.107023105);
    pub const INDIGO: Color = cc(0.070360094, 0.0, 0.22322796);
    pub const IVORY: Color = cc(1.0, 1.0, 0.8713671);
    pub const KHAKI: Color = cc(0.8713671, 0.7912979, 0.26225066);
    pub const LAVENDER: Color = cc(0.7912979, 0.7912979, 0.9559733);
    pub const LAVENDER_BLUSH: Color = cc(1.0, 0.8713671, 0.91309863);
    pub const LAWN_GREEN: Color = cc(0.20155625, 0.9734453, 0.0);
    pub const LEMON_CHIFFON: Color = cc(1.0, 0.9559733, 0.61049557);
    pub const LIGHT_BLUE: Color = cc(0.41788507, 0.6866853, 0.7912979);
    pub const LIGHT_CORAL: Color = cc(0.8713671, 0.2158605, 0.2158605);
    pub const LIGHT_CYAN: Color = cc(0.7454042, 1.0, 1.0);
    pub const LIGHT_GOLDEN_ROD_YELLOW: Color =
        cc(0.9559733, 0.9559733, 0.6444797);
    pub const LIGHT_GRAY: Color = cc(0.65140563, 0.65140563, 0.65140563);
    pub const LIGHT_GREY: Color = cc(0.65140563, 0.65140563, 0.65140563);
    pub const LIGHT_GREEN: Color = cc(0.27889428, 0.8549926, 0.27889428);
    pub const LIGHT_PINK: Color = cc(1.0, 0.4677838, 0.5332764);
    pub const LIGHT_SALMON: Color = cc(1.0, 0.3515326, 0.19461784);
    pub const LIGHT_SEA_GREEN: Color = cc(0.014443844, 0.4452012, 0.40197778);
    pub const LIGHT_SKY_BLUE: Color = cc(0.24228112, 0.6172066, 0.9559733);
    pub const LIGHT_SLATE_GRAY: Color = cc(0.18447499, 0.24620132, 0.31854677);
    pub const LIGHT_SLATE_GREY: Color = cc(0.18447499, 0.24620132, 0.31854677);
    pub const LIGHT_STEEL_BLUE: Color = cc(0.43415365, 0.55201143, 0.73046076);
    pub const LIGHT_YELLOW: Color = cc(1.0, 1.0, 0.7454042);
    pub const LIME: Color = cc(0.0, 1.0, 0.0);
    pub const LIME_GREEN: Color = cc(0.031896032, 0.61049557, 0.031896032);
    pub const LINEN: Color = cc(0.9559733, 0.8713671, 0.7912979);
    pub const MAGENTA: Color = cc(1.0, 0.0, 1.0);
    pub const MAROON: Color = cc(0.2158605, 0.0, 0.0);
    pub const MEDIUM_AQUA_MARINE: Color =
        cc(0.13286832, 0.61049557, 0.40197778);
    pub const MEDIUM_BLUE: Color = cc(0.0, 0.0, 0.61049557);
    pub const MEDIUM_ORCHID: Color = cc(0.49102086, 0.09084171, 0.65140563);
    pub const MEDIUM_PURPLE: Color = cc(0.29177064, 0.16202937, 0.70837575);
    pub const MEDIUM_SEA_GREEN: Color = cc(0.045186203, 0.4507858, 0.1651322);
    pub const MEDIUM_SLATE_BLUE: Color = cc(0.19806932, 0.13843161, 0.8549926);
    pub const MEDIUM_SPRING_GREEN: Color = cc(0.0, 0.9559733, 0.3231432);
    pub const MEDIUM_TURQUOISE: Color =
        cc(0.064803265, 0.63759685, 0.60382736);
    pub const MEDIUM_VIOLET_RED: Color =
        cc(0.57112485, 0.007499032, 0.23455058);
    pub const MIDNIGHT_BLUE: Color = cc(0.009721218, 0.009721218, 0.16202937);
    pub const MINT_CREAM: Color = cc(0.91309863, 1.0, 0.9559733);
    pub const MISTY_ROSE: Color = cc(1.0, 0.7758222, 0.7529422);
    pub const MOCCASIN: Color = cc(1.0, 0.7758222, 0.462077);
    pub const NAVAJO_WHITE: Color = cc(1.0, 0.73046076, 0.41788507);
    pub const NAVY: Color = cc(0.0, 0.0, 0.2158605);
    pub const OLD_LACE: Color = cc(0.9822506, 0.91309863, 0.7912979);
    pub const OLIVE: Color = cc(0.2158605, 0.2158605, 0.0);
    pub const OLIVE_DRAB: Color = cc(0.14702727, 0.2704978, 0.016807375);
    pub const ORANGE: Color = cc(1.0, 0.37626213, 0.0);
    pub const ORANGE_RED: Color = cc(1.0, 0.059511237, 0.0);
    pub const ORCHID: Color = cc(0.7011019, 0.16202937, 0.67244315);
    pub const PALE_GOLDEN_ROD: Color = cc(0.8549926, 0.80695224, 0.40197778);
    pub const PALE_GREEN: Color = cc(0.31398872, 0.9646863, 0.31398872);
    pub const PALE_TURQUOISE: Color = cc(0.4286905, 0.8549926, 0.8549926);
    pub const PALE_VIOLET_RED: Color = cc(0.70837575, 0.16202937, 0.29177064);
    pub const PAPAYA_WHIP: Color = cc(1.0, 0.8631572, 0.6653873);
    pub const PEACH_PUFF: Color = cc(1.0, 0.7011019, 0.48514995);
    pub const PERU: Color = cc(0.61049557, 0.23455058, 0.049706567);
    pub const PINK: Color = cc(1.0, 0.5271151, 0.59720176);
    pub const PLUM: Color = cc(0.7230551, 0.3515326, 0.7230551);
    pub const POWDER_BLUE: Color = cc(0.43415365, 0.7454042, 0.7912979);
    pub const PURPLE: Color = cc(0.2158605, 0.0, 0.2158605);
    pub const REBECCA_PURPLE: Color = cc(0.13286832, 0.033104766, 0.31854677);
    pub const RED: Color = cc(1.0, 0.0, 0.0);
    pub const ROSY_BROWN: Color = cc(0.5028865, 0.2746773, 0.2746773);
    pub const ROYAL_BLUE: Color = cc(0.052860647, 0.14126329, 0.7529422);
    pub const SADDLE_BROWN: Color = cc(0.25818285, 0.059511237, 0.0065120906);
    pub const SALMON: Color = cc(0.9559733, 0.2158605, 0.1682694);
    pub const SANDY_BROWN: Color = cc(0.9046612, 0.3712377, 0.116970666);
    pub const SEA_GREEN: Color = cc(0.027320892, 0.25818285, 0.09530747);
    pub const SEA_SHELL: Color = cc(1.0, 0.91309863, 0.8549926);
    pub const SIENNA: Color = cc(0.3515326, 0.08437621, 0.026241222);
    pub const SILVER: Color = cc(0.5271151, 0.5271151, 0.5271151);
    pub const SKY_BLUE: Color = cc(0.24228112, 0.6172066, 0.8307699);
    pub const SLATE_BLUE: Color = cc(0.14412847, 0.10224173, 0.61049557);
    pub const SLATE_GRAY: Color = cc(0.16202937, 0.2158605, 0.27889428);
    pub const SLATE_GREY: Color = cc(0.16202937, 0.2158605, 0.27889428);
    pub const SNOW: Color = cc(1.0, 0.9559733, 0.9559733);
    pub const SPRING_GREEN: Color = cc(0.0, 1.0, 0.21223076);
    pub const STEEL_BLUE: Color = cc(0.061246052, 0.22322796, 0.45641103);
    pub const TAN: Color = cc(0.6444797, 0.45641103, 0.26225066);
    pub const TEAL: Color = cc(0.0, 0.2158605, 0.2158605);
    pub const THISTLE: Color = cc(0.6866853, 0.52099556, 0.6866853);
    pub const TOMATO: Color = cc(1.0, 0.12477182, 0.063010015);
    pub const TURQUOISE: Color = cc(0.051269457, 0.7454042, 0.63075715);
    pub const VIOLET: Color = cc(0.8549926, 0.22322796, 0.8549926);
    pub const WHEAT: Color = cc(0.91309863, 0.73046076, 0.4507858);
    pub const WHITE: Color = cc(1.0, 1.0, 1.0);
    pub const WHITE_SMOKE: Color = cc(0.91309863, 0.91309863, 0.91309863);
    pub const YELLOW: Color = cc(1.0, 1.0, 0.0);
    pub const YELLOW_GREEN: Color = cc(0.3231432, 0.61049557, 0.031896032);
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
