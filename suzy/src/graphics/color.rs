/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::fmt::{self, LowerHex, UpperHex};

use crate::animation::{Lerp, LerpDistance};

/// A type which represents a color with 32 bit floating point components.
///
/// Colors can be crated a number of ways:
///
/// ```rust
/// # use suzy::graphics::Color;
/// let color0 = Color::create_rgb(0.93333334, 0.50980395, 0.93333334);
/// let color1 = Color::create_rgb8(238, 130, 238);
/// let color2: Color = "#EE82EE".parse().unwrap();
/// assert_eq!(color0, color1);
/// assert_eq!(color1, color2);
/// ```
///
/// Colors can be formatted with hex style to get familar results:
/// ```rust
/// # use suzy::graphics::Color;
/// let color = Color::create_rgb8(238, 130, 238);
/// let string = format!("{:X}", color);
/// assert_eq!(string, "#EE82EEFF");
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

const MAX8: f32 = std::u8::MAX as f32;

impl Color {
    /// Create a new color with the specified RGBA components, where 1.0
    /// represents the maximum for that component.
    pub const fn create_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    /// Create a new color with the specified RGBA components, where 255
    /// represents the maximum for that component.
    pub fn create_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color {
            r: (r as f32) / MAX8,
            g: (g as f32) / MAX8,
            b: (b as f32) / MAX8,
            a: (a as f32) / MAX8,
        }
    }

    /// Create a new color with the specified RGB components, where 1.0
    /// represents the maximum for that component.  The alpha is assumed to
    /// be fully opaque.
    pub const fn create_rgb(r: f32, g: f32, b: f32) -> Self {
        Self::create_rgba(r, g, b, 1.0)
    }

    /// Create a new color with the specified RGB components, where 255
    /// represents the maximum for that component.  The alpha is assumed to
    /// be fully opaque.
    pub fn create_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::create_rgba8(r, g, b, u8::max_value())
    }

    /// Get the RGBA components of this color, as floats, where 1.0
    /// represents the maximum for that component.
    pub fn rgba(&self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }

    /// Get the RGBA components of this color, as integers, where 255
    /// represents the maximum for that component.
    pub fn rgba8(&self) -> [u8; 4] {
        [
            (self.r * MAX8).round() as u8,
            (self.g * MAX8).round() as u8,
            (self.b * MAX8).round() as u8,
            (self.a * MAX8).round() as u8,
        ]
    }

    /// Apply a tint to this color based on another color.
    pub fn tint(&mut self, other: Self) {
        self.r *= other.r;
        self.g *= other.g;
        self.b *= other.b;
        self.a *= other.a;
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

impl From<u32> for Color {
    /// Create a color from values packed in a 32 bit integer, assuming ARGB.
    #[inline]
    fn from(code: u32) -> Self {
        let array = code.to_be_bytes();
        Color::create_rgba8(array[1], array[2], array[3], array[0])
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
                Ok(Self::create_rgba8(bytes[0], bytes[1], bytes[2], bytes[3]))
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

impl Color {
    /// Color constant 'ALICE_BLUE'
    pub const ALICE_BLUE: Color = Color::create_rgb(0.9411765, 0.972549, 1.0);
    /// Color constant 'ANTIQUE_WHITE'
    pub const ANTIQUE_WHITE: Color =
        Color::create_rgb(0.9803922, 0.9215686, 0.8431373);
    /// Color constant 'AQUA'
    pub const AQUA: Color = Color::create_rgb(0.0, 1.0, 1.0);
    /// Color constant 'AQUAMARINE'
    pub const AQUAMARINE: Color = Color::create_rgb(0.4980392, 1.0, 0.8313725);
    /// Color constant 'AZURE'
    pub const AZURE: Color = Color::create_rgb(0.9411765, 1.0, 1.0);
    /// Color constant 'BEIGE'
    pub const BEIGE: Color =
        Color::create_rgb(0.9607843, 0.9607843, 0.8627451);
    /// Color constant 'BISQUE'
    pub const BISQUE: Color = Color::create_rgb(1.0, 0.8941176, 0.7686275);
    /// Color constant 'BLACK'
    pub const BLACK: Color = Color::create_rgb(0.0, 0.0, 0.0);
    /// Color constant 'BLANCHED_ALMOND'
    pub const BLANCHED_ALMOND: Color =
        Color::create_rgb(1.0, 0.9215686, 0.8039216);
    /// Color constant 'BLUE'
    pub const BLUE: Color = Color::create_rgb(0.0, 0.0, 1.0);
    /// Color constant 'BLUE_VIOLET'
    pub const BLUE_VIOLET: Color =
        Color::create_rgb(0.5411765, 0.1686275, 0.8862745);
    /// Color constant 'BROWN'
    pub const BROWN: Color =
        Color::create_rgb(0.6470588, 0.1647059, 0.1647059);
    /// Color constant 'BURLY_WOOD'
    pub const BURLY_WOOD: Color =
        Color::create_rgb(0.8705882, 0.7215686, 0.5294118);
    /// Color constant 'CADET_BLUE'
    pub const CADET_BLUE: Color =
        Color::create_rgb(0.372549, 0.6196078, 0.627451);
    /// Color constant 'CHARTREUSE'
    pub const CHARTREUSE: Color = Color::create_rgb(0.4980392, 1.0, 0.0);
    /// Color constant 'CHOCOLATE'
    pub const CHOCOLATE: Color =
        Color::create_rgb(0.8235294, 0.4117647, 0.1176471);
    /// Color constant 'CORAL'
    pub const CORAL: Color = Color::create_rgb(1.0, 0.4980392, 0.3137255);
    /// Color constant 'CORNFLOWER_BLUE'
    pub const CORNFLOWER_BLUE: Color =
        Color::create_rgb(0.3921569, 0.5843137, 0.9294118);
    /// Color constant 'CORNSILK'
    pub const CORNSILK: Color = Color::create_rgb(1.0, 0.972549, 0.8627451);
    /// Color constant 'CRIMSON'
    pub const CRIMSON: Color =
        Color::create_rgb(0.8627451, 0.0784314, 0.2352941);
    /// Color constant 'CYAN'
    pub const CYAN: Color = Color::create_rgb(0.0, 1.0, 1.0);
    /// Color constant 'DARK_BLUE'
    pub const DARK_BLUE: Color = Color::create_rgb(0.0, 0.0, 0.545098);
    /// Color constant 'DARK_CYAN'
    pub const DARK_CYAN: Color = Color::create_rgb(0.0, 0.545098, 0.545098);
    /// Color constant 'DARK_GOLDEN_ROD'
    pub const DARK_GOLDEN_ROD: Color =
        Color::create_rgb(0.7215686, 0.5254902, 0.0431373);
    /// Color constant 'DARK_GRAY'
    pub const DARK_GRAY: Color =
        Color::create_rgb(0.6627451, 0.6627451, 0.6627451);
    /// Color constant 'DARK_GREY'
    pub const DARK_GREY: Color =
        Color::create_rgb(0.6627451, 0.6627451, 0.6627451);
    /// Color constant 'DARK_GREEN'
    pub const DARK_GREEN: Color = Color::create_rgb(0.0, 0.3921569, 0.0);
    /// Color constant 'DARK_KHAKI'
    pub const DARK_KHAKI: Color =
        Color::create_rgb(0.7411765, 0.7176471, 0.4196078);
    /// Color constant 'DARK_MAGENTA'
    pub const DARK_MAGENTA: Color = Color::create_rgb(0.545098, 0.0, 0.545098);
    /// Color constant 'DARK_OLIVE_GREEN'
    pub const DARK_OLIVE_GREEN: Color =
        Color::create_rgb(0.3333333, 0.4196078, 0.1843137);
    /// Color constant 'DARK_ORANGE'
    pub const DARK_ORANGE: Color = Color::create_rgb(1.0, 0.5490196, 0.0);
    /// Color constant 'DARK_ORCHID'
    pub const DARK_ORCHID: Color = Color::create_rgb(0.6, 0.1960784, 0.8);
    /// Color constant 'DARK_RED'
    pub const DARK_RED: Color = Color::create_rgb(0.545098, 0.0, 0.0);
    /// Color constant 'DARK_SALMON'
    pub const DARK_SALMON: Color =
        Color::create_rgb(0.9137255, 0.5882353, 0.4784314);
    /// Color constant 'DARK_SEA_GREEN'
    pub const DARK_SEA_GREEN: Color =
        Color::create_rgb(0.5607843, 0.7372549, 0.5607843);
    /// Color constant 'DARK_SLATE_BLUE'
    pub const DARK_SLATE_BLUE: Color =
        Color::create_rgb(0.2823529, 0.2392157, 0.545098);
    /// Color constant 'DARK_SLATE_GRAY'
    pub const DARK_SLATE_GRAY: Color =
        Color::create_rgb(0.1843137, 0.3098039, 0.3098039);
    /// Color constant 'DARK_SLATE_GREY'
    pub const DARK_SLATE_GREY: Color =
        Color::create_rgb(0.1843137, 0.3098039, 0.3098039);
    /// Color constant 'DARK_TURQUOISE'
    pub const DARK_TURQUOISE: Color =
        Color::create_rgb(0.0, 0.8078431, 0.8196078);
    /// Color constant 'DARK_VIOLET'
    pub const DARK_VIOLET: Color = Color::create_rgb(0.5803922, 0.0, 0.827451);
    /// Color constant 'DEEP_PINK'
    pub const DEEP_PINK: Color = Color::create_rgb(1.0, 0.0784314, 0.5764706);
    /// Color constant 'DEEP_SKY_BLUE'
    pub const DEEP_SKY_BLUE: Color = Color::create_rgb(0.0, 0.7490196, 1.0);
    /// Color constant 'DIM_GRAY'
    pub const DIM_GRAY: Color =
        Color::create_rgb(0.4117647, 0.4117647, 0.4117647);
    /// Color constant 'DIM_GREY'
    pub const DIM_GREY: Color =
        Color::create_rgb(0.4117647, 0.4117647, 0.4117647);
    /// Color constant 'DODGER_BLUE'
    pub const DODGER_BLUE: Color =
        Color::create_rgb(0.1176471, 0.5647059, 1.0);
    /// Color constant 'FIRE_BRICK'
    pub const FIRE_BRICK: Color =
        Color::create_rgb(0.6980392, 0.1333333, 0.1333333);
    /// Color constant 'FLORAL_WHITE'
    pub const FLORAL_WHITE: Color =
        Color::create_rgb(1.0, 0.9803922, 0.9411765);
    /// Color constant 'FOREST_GREEN'
    pub const FOREST_GREEN: Color =
        Color::create_rgb(0.1333333, 0.545098, 0.1333333);
    /// Color constant 'FUCHSIA'
    pub const FUCHSIA: Color = Color::create_rgb(1.0, 0.0, 1.0);
    /// Color constant 'GAINSBORO'
    pub const GAINSBORO: Color =
        Color::create_rgb(0.8627451, 0.8627451, 0.8627451);
    /// Color constant 'GHOST_WHITE'
    pub const GHOST_WHITE: Color = Color::create_rgb(0.972549, 0.972549, 1.0);
    /// Color constant 'GOLD'
    pub const GOLD: Color = Color::create_rgb(1.0, 0.8431373, 0.0);
    /// Color constant 'GOLDEN_ROD'
    pub const GOLDEN_ROD: Color =
        Color::create_rgb(0.854902, 0.6470588, 0.1254902);
    /// Color constant 'GRAY'
    pub const GRAY: Color = Color::create_rgb(0.5019608, 0.5019608, 0.5019608);
    /// Color constant 'GREY'
    pub const GREY: Color = Color::create_rgb(0.5019608, 0.5019608, 0.5019608);
    /// Color constant 'GREEN'
    pub const GREEN: Color = Color::create_rgb(0.0, 0.5019608, 0.0);
    /// Color constant 'GREEN_YELLOW'
    pub const GREEN_YELLOW: Color =
        Color::create_rgb(0.6784314, 1.0, 0.1843137);
    /// Color constant 'HONEY_DEW'
    pub const HONEY_DEW: Color = Color::create_rgb(0.9411765, 1.0, 0.9411765);
    /// Color constant 'HOT_PINK'
    pub const HOT_PINK: Color = Color::create_rgb(1.0, 0.4117647, 0.7058824);
    /// Color constant 'INDIAN_RED'
    pub const INDIAN_RED: Color =
        Color::create_rgb(0.8039216, 0.3607843, 0.3607843);
    /// Color constant 'INDIGO'
    pub const INDIGO: Color = Color::create_rgb(0.2941176, 0.0, 0.5098039);
    /// Color constant 'IVORY'
    pub const IVORY: Color = Color::create_rgb(1.0, 1.0, 0.9411765);
    /// Color constant 'KHAKI'
    pub const KHAKI: Color =
        Color::create_rgb(0.9411765, 0.9019608, 0.5490196);
    /// Color constant 'LAVENDER'
    pub const LAVENDER: Color =
        Color::create_rgb(0.9019608, 0.9019608, 0.9803922);
    /// Color constant 'LAVENDER_BLUSH'
    pub const LAVENDER_BLUSH: Color =
        Color::create_rgb(1.0, 0.9411765, 0.9607843);
    /// Color constant 'LAWN_GREEN'
    pub const LAWN_GREEN: Color = Color::create_rgb(0.4862745, 0.9882353, 0.0);
    /// Color constant 'LEMON_CHIFFON'
    pub const LEMON_CHIFFON: Color =
        Color::create_rgb(1.0, 0.9803922, 0.8039216);
    /// Color constant 'LIGHT_BLUE'
    pub const LIGHT_BLUE: Color =
        Color::create_rgb(0.6784314, 0.8470588, 0.9019608);
    /// Color constant 'LIGHT_CORAL'
    pub const LIGHT_CORAL: Color =
        Color::create_rgb(0.9411765, 0.5019608, 0.5019608);
    /// Color constant 'LIGHT_CYAN'
    pub const LIGHT_CYAN: Color = Color::create_rgb(0.8784314, 1.0, 1.0);
    /// Color constant 'LIGHT_GOLDEN_ROD_YELLOW'
    pub const LIGHT_GOLDEN_ROD_YELLOW: Color =
        Color::create_rgb(0.9803922, 0.9803922, 0.8235294);
    /// Color constant 'LIGHT_GRAY'
    pub const LIGHT_GRAY: Color =
        Color::create_rgb(0.827451, 0.827451, 0.827451);
    /// Color constant 'LIGHT_GREY'
    pub const LIGHT_GREY: Color =
        Color::create_rgb(0.827451, 0.827451, 0.827451);
    /// Color constant 'LIGHT_GREEN'
    pub const LIGHT_GREEN: Color =
        Color::create_rgb(0.5647059, 0.9333333, 0.5647059);
    /// Color constant 'LIGHT_PINK'
    pub const LIGHT_PINK: Color = Color::create_rgb(1.0, 0.7137255, 0.7568627);
    /// Color constant 'LIGHT_SALMON'
    pub const LIGHT_SALMON: Color =
        Color::create_rgb(1.0, 0.627451, 0.4784314);
    /// Color constant 'LIGHT_SEA_GREEN'
    pub const LIGHT_SEA_GREEN: Color =
        Color::create_rgb(0.1254902, 0.6980392, 0.6666667);
    /// Color constant 'LIGHT_SKY_BLUE'
    pub const LIGHT_SKY_BLUE: Color =
        Color::create_rgb(0.5294118, 0.8078431, 0.9803922);
    /// Color constant 'LIGHT_SLATE_GRAY'
    pub const LIGHT_SLATE_GRAY: Color =
        Color::create_rgb(0.4666667, 0.5333333, 0.6);
    /// Color constant 'LIGHT_SLATE_GREY'
    pub const LIGHT_SLATE_GREY: Color =
        Color::create_rgb(0.4666667, 0.5333333, 0.6);
    /// Color constant 'LIGHT_STEEL_BLUE'
    pub const LIGHT_STEEL_BLUE: Color =
        Color::create_rgb(0.6901961, 0.7686275, 0.8705882);
    /// Color constant 'LIGHT_YELLOW'
    pub const LIGHT_YELLOW: Color = Color::create_rgb(1.0, 1.0, 0.8784314);
    /// Color constant 'LIME'
    pub const LIME: Color = Color::create_rgb(0.0, 1.0, 0.0);
    /// Color constant 'LIME_GREEN'
    pub const LIME_GREEN: Color =
        Color::create_rgb(0.1960784, 0.8039216, 0.1960784);
    /// Color constant 'LINEN'
    pub const LINEN: Color =
        Color::create_rgb(0.9803922, 0.9411765, 0.9019608);
    /// Color constant 'MAGENTA'
    pub const MAGENTA: Color = Color::create_rgb(1.0, 0.0, 1.0);
    /// Color constant 'MAROON'
    pub const MAROON: Color = Color::create_rgb(0.5019608, 0.0, 0.0);
    /// Color constant 'MEDIUM_AQUA_MARINE'
    pub const MEDIUM_AQUA_MARINE: Color =
        Color::create_rgb(0.4, 0.8039216, 0.6666667);
    /// Color constant 'MEDIUM_BLUE'
    pub const MEDIUM_BLUE: Color = Color::create_rgb(0.0, 0.0, 0.8039216);
    /// Color constant 'MEDIUM_ORCHID'
    pub const MEDIUM_ORCHID: Color =
        Color::create_rgb(0.7294118, 0.3333333, 0.827451);
    /// Color constant 'MEDIUM_PURPLE'
    pub const MEDIUM_PURPLE: Color =
        Color::create_rgb(0.5764706, 0.4392157, 0.8588235);
    /// Color constant 'MEDIUM_SEA_GREEN'
    pub const MEDIUM_SEA_GREEN: Color =
        Color::create_rgb(0.2352941, 0.7019608, 0.4431373);
    /// Color constant 'MEDIUM_SLATE_BLUE'
    pub const MEDIUM_SLATE_BLUE: Color =
        Color::create_rgb(0.4823529, 0.4078431, 0.9333333);
    /// Color constant 'MEDIUM_SPRING_GREEN'
    pub const MEDIUM_SPRING_GREEN: Color =
        Color::create_rgb(0.0, 0.9803922, 0.6039216);
    /// Color constant 'MEDIUM_TURQUOISE'
    pub const MEDIUM_TURQUOISE: Color =
        Color::create_rgb(0.2823529, 0.8196078, 0.8);
    /// Color constant 'MEDIUM_VIOLET_RED'
    pub const MEDIUM_VIOLET_RED: Color =
        Color::create_rgb(0.7803922, 0.0823529, 0.5215686);
    /// Color constant 'MIDNIGHT_BLUE'
    pub const MIDNIGHT_BLUE: Color =
        Color::create_rgb(0.0980392, 0.0980392, 0.4392157);
    /// Color constant 'MINT_CREAM'
    pub const MINT_CREAM: Color = Color::create_rgb(0.9607843, 1.0, 0.9803922);
    /// Color constant 'MISTY_ROSE'
    pub const MISTY_ROSE: Color = Color::create_rgb(1.0, 0.8941176, 0.8823529);
    /// Color constant 'MOCCASIN'
    pub const MOCCASIN: Color = Color::create_rgb(1.0, 0.8941176, 0.7098039);
    /// Color constant 'NAVAJO_WHITE'
    pub const NAVAJO_WHITE: Color =
        Color::create_rgb(1.0, 0.8705882, 0.6784314);
    /// Color constant 'NAVY'
    pub const NAVY: Color = Color::create_rgb(0.0, 0.0, 0.5019608);
    /// Color constant 'OLD_LACE'
    pub const OLD_LACE: Color =
        Color::create_rgb(0.9921569, 0.9607843, 0.9019608);
    /// Color constant 'OLIVE'
    pub const OLIVE: Color = Color::create_rgb(0.5019608, 0.5019608, 0.0);
    /// Color constant 'OLIVE_DRAB'
    pub const OLIVE_DRAB: Color =
        Color::create_rgb(0.4196078, 0.5568627, 0.1372549);
    /// Color constant 'ORANGE'
    pub const ORANGE: Color = Color::create_rgb(1.0, 0.6470588, 0.0);
    /// Color constant 'ORANGE_RED'
    pub const ORANGE_RED: Color = Color::create_rgb(1.0, 0.2705882, 0.0);
    /// Color constant 'ORCHID'
    pub const ORCHID: Color =
        Color::create_rgb(0.854902, 0.4392157, 0.8392157);
    /// Color constant 'PALE_GOLDEN_ROD'
    pub const PALE_GOLDEN_ROD: Color =
        Color::create_rgb(0.9333333, 0.9098039, 0.6666667);
    /// Color constant 'PALE_GREEN'
    pub const PALE_GREEN: Color =
        Color::create_rgb(0.5960784, 0.9843137, 0.5960784);
    /// Color constant 'PALE_TURQUOISE'
    pub const PALE_TURQUOISE: Color =
        Color::create_rgb(0.6862745, 0.9333333, 0.9333333);
    /// Color constant 'PALE_VIOLET_RED'
    pub const PALE_VIOLET_RED: Color =
        Color::create_rgb(0.8588235, 0.4392157, 0.5764706);
    /// Color constant 'PAPAYA_WHIP'
    pub const PAPAYA_WHIP: Color =
        Color::create_rgb(1.0, 0.9372549, 0.8352941);
    /// Color constant 'PEACH_PUFF'
    pub const PEACH_PUFF: Color = Color::create_rgb(1.0, 0.854902, 0.7254902);
    /// Color constant 'PERU'
    pub const PERU: Color = Color::create_rgb(0.8039216, 0.5215686, 0.2470588);
    /// Color constant 'PINK'
    pub const PINK: Color = Color::create_rgb(1.0, 0.7529412, 0.7960784);
    /// Color constant 'PLUM'
    pub const PLUM: Color = Color::create_rgb(0.8666667, 0.627451, 0.8666667);
    /// Color constant 'POWDER_BLUE'
    pub const POWDER_BLUE: Color =
        Color::create_rgb(0.6901961, 0.8784314, 0.9019608);
    /// Color constant 'PURPLE'
    pub const PURPLE: Color = Color::create_rgb(0.5019608, 0.0, 0.5019608);
    /// Color constant 'REBECCA_PURPLE'
    pub const REBECCA_PURPLE: Color = Color::create_rgb(0.4, 0.2, 0.6);
    /// Color constant 'RED'
    pub const RED: Color = Color::create_rgb(1.0, 0.0, 0.0);
    /// Color constant 'ROSY_BROWN'
    pub const ROSY_BROWN: Color =
        Color::create_rgb(0.7372549, 0.5607843, 0.5607843);
    /// Color constant 'ROYAL_BLUE'
    pub const ROYAL_BLUE: Color =
        Color::create_rgb(0.254902, 0.4117647, 0.8823529);
    /// Color constant 'SADDLE_BROWN'
    pub const SADDLE_BROWN: Color =
        Color::create_rgb(0.545098, 0.2705882, 0.0745098);
    /// Color constant 'SALMON'
    pub const SALMON: Color =
        Color::create_rgb(0.9803922, 0.5019608, 0.4470588);
    /// Color constant 'SANDY_BROWN'
    pub const SANDY_BROWN: Color =
        Color::create_rgb(0.9568627, 0.6431373, 0.3764706);
    /// Color constant 'SEA_GREEN'
    pub const SEA_GREEN: Color =
        Color::create_rgb(0.1803922, 0.545098, 0.3411765);
    /// Color constant 'SEA_SHELL'
    pub const SEA_SHELL: Color = Color::create_rgb(1.0, 0.9607843, 0.9333333);
    /// Color constant 'SIENNA'
    pub const SIENNA: Color =
        Color::create_rgb(0.627451, 0.3215686, 0.1764706);
    /// Color constant 'SILVER'
    pub const SILVER: Color =
        Color::create_rgb(0.7529412, 0.7529412, 0.7529412);
    /// Color constant 'SKY_BLUE'
    pub const SKY_BLUE: Color =
        Color::create_rgb(0.5294118, 0.8078431, 0.9215686);
    /// Color constant 'SLATE_BLUE'
    pub const SLATE_BLUE: Color =
        Color::create_rgb(0.4156863, 0.3529412, 0.8039216);
    /// Color constant 'SLATE_GRAY'
    pub const SLATE_GRAY: Color =
        Color::create_rgb(0.4392157, 0.5019608, 0.5647059);
    /// Color constant 'SLATE_GREY'
    pub const SLATE_GREY: Color =
        Color::create_rgb(0.4392157, 0.5019608, 0.5647059);
    /// Color constant 'SNOW'
    pub const SNOW: Color = Color::create_rgb(1.0, 0.9803922, 0.9803922);
    /// Color constant 'SPRING_GREEN'
    pub const SPRING_GREEN: Color = Color::create_rgb(0.0, 1.0, 0.4980392);
    /// Color constant 'STEEL_BLUE'
    pub const STEEL_BLUE: Color =
        Color::create_rgb(0.2745098, 0.5098039, 0.7058824);
    /// Color constant 'TAN'
    pub const TAN: Color = Color::create_rgb(0.8235294, 0.7058824, 0.5490196);
    /// Color constant 'TEAL'
    pub const TEAL: Color = Color::create_rgb(0.0, 0.5019608, 0.5019608);
    /// Color constant 'THISTLE'
    pub const THISTLE: Color =
        Color::create_rgb(0.8470588, 0.7490196, 0.8470588);
    /// Color constant 'TOMATO'
    pub const TOMATO: Color = Color::create_rgb(1.0, 0.3882353, 0.2784314);
    /// Color constant 'TURQUOISE'
    pub const TURQUOISE: Color =
        Color::create_rgb(0.2509804, 0.8784314, 0.8156863);
    /// Color constant 'VIOLET'
    pub const VIOLET: Color =
        Color::create_rgb(0.93333334, 0.50980395, 0.93333334);
    /// Color constant 'WHEAT'
    pub const WHEAT: Color =
        Color::create_rgb(0.9607843, 0.8705882, 0.7019608);
    /// Color constant 'WHITE'
    pub const WHITE: Color = Color::create_rgb(1.0, 1.0, 1.0);
    /// Color constant 'WHITE_SMOKE'
    pub const WHITE_SMOKE: Color =
        Color::create_rgb(0.9607843, 0.9607843, 0.9607843);
    /// Color constant 'YELLOW'
    pub const YELLOW: Color = Color::create_rgb(1.0, 1.0, 0.0);
    /// Color constant 'YELLOW_GREEN'
    pub const YELLOW_GREEN: Color =
        Color::create_rgb(0.6039216, 0.8039216, 0.1960784);

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
