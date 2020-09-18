/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::{self, LowerHex, UpperHex};

use crate::animation::{
    Lerp,
    LerpDistance,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

const MAX8: f32 = std::u8::MAX as f32;

impl Color {
    pub const fn create_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub fn create_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color {
            r: (r as f32) / MAX8,
            g: (g as f32) / MAX8,
            b: (b as f32) / MAX8,
            a: (a as f32) / MAX8,
        }
    }

    pub const fn create_rgb(r: f32, g: f32, b: f32) -> Self {
        Self::create_rgba(r, g, b, 1.0)
    }

    pub fn create_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::create_rgba8(r, g, b, u8::max_value())
    }

    pub fn rgba(&self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }

    pub fn rgba8(&self) -> (u8, u8, u8, u8) {
        (
            (self.r * MAX8) as u8,
            (self.g * MAX8) as u8,
            (self.b * MAX8) as u8,
            (self.a * MAX8) as u8,
        )
    }

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
            + (a.a - b.a).powi(2)
        )
    }
}

impl From<u32> for Color {
    #[inline]
    fn from(code: u32) -> Self {
        let array = code.to_be_bytes();
        Color::create_rgba8(array[1], array[2], array[3], array[0])
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ParseColorError;

impl From<std::num::ParseIntError> for ParseColorError {
    fn from(_orig: std::num::ParseIntError) -> Self { Self }
}

impl std::str::FromStr for Color {
    type Err = ParseColorError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(hex_part) = s.strip_prefix('#') {
            if hex_part.len() == 6 || hex_part.len() == 8 {
                let mut int = u32::from_str_radix(hex_part, 16)?;
                if hex_part.len() == 6 {
                    int <<= 8;
                }
                Ok(int.into())
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
        let rgba = self.rgba8();
        let bytes = [rgba.0, rgba.1, rgba.2, rgba.3];
        write!(f, "#{:08x}", u32::from_be_bytes(bytes))
    }
}

impl UpperHex for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rgba = self.rgba8();
        let bytes = [rgba.0, rgba.1, rgba.2, rgba.3];
        f.write_str("#")?;
        UpperHex::fmt(&u32::from_be_bytes(bytes), f)?;
        Ok(())
    }
}

impl Color {
    pub const ALICE_BLUE: Color = Color::create_rgb(0.9411764705882353, 0.9725490196078431, 1.0);
    pub const ANTIQUE_WHITE: Color = Color::create_rgb(0.9803921568627451, 0.9215686274509803, 0.8431372549019608);
    pub const AQUA: Color = Color::create_rgb(0.0, 1.0, 1.0);
    pub const AQUAMARINE: Color = Color::create_rgb(0.4980392156862745, 1.0, 0.8313725490196079);
    pub const AZURE: Color = Color::create_rgb(0.9411764705882353, 1.0, 1.0);
    pub const BEIGE: Color = Color::create_rgb(0.9607843137254902, 0.9607843137254902, 0.8627450980392157);
    pub const BISQUE: Color = Color::create_rgb(1.0, 0.8941176470588236, 0.7686274509803922);
    pub const BLACK: Color = Color::create_rgb(0.0, 0.0, 0.0);
    pub const BLANCHED_ALMOND: Color = Color::create_rgb(1.0, 0.9215686274509803, 0.803921568627451);
    pub const BLUE: Color = Color::create_rgb(0.0, 0.0, 1.0);
    pub const BLUE_VIOLET: Color = Color::create_rgb(0.5411764705882353, 0.16862745098039217, 0.8862745098039215);
    pub const BROWN: Color = Color::create_rgb(0.6470588235294118, 0.16470588235294117, 0.16470588235294117);
    pub const BURLY_WOOD: Color = Color::create_rgb(0.8705882352941177, 0.7215686274509804, 0.5294117647058824);
    pub const CADET_BLUE: Color = Color::create_rgb(0.37254901960784315, 0.6196078431372549, 0.6274509803921569);
    pub const CHARTREUSE: Color = Color::create_rgb(0.4980392156862745, 1.0, 0.0);
    pub const CHOCOLATE: Color = Color::create_rgb(0.8235294117647058, 0.4117647058823529, 0.11764705882352941);
    pub const CORAL: Color = Color::create_rgb(1.0, 0.4980392156862745, 0.3137254901960784);
    pub const CORNFLOWER_BLUE: Color = Color::create_rgb(0.39215686274509803, 0.5843137254901961, 0.9294117647058824);
    pub const CORNSILK: Color = Color::create_rgb(1.0, 0.9725490196078431, 0.8627450980392157);
    pub const CRIMSON: Color = Color::create_rgb(0.8627450980392157, 0.0784313725490196, 0.23529411764705882);
    pub const CYAN: Color = Color::create_rgb(0.0, 1.0, 1.0);
    pub const DARK_BLUE: Color = Color::create_rgb(0.0, 0.0, 0.5450980392156862);
    pub const DARK_CYAN: Color = Color::create_rgb(0.0, 0.5450980392156862, 0.5450980392156862);
    pub const DARK_GOLDEN_ROD: Color = Color::create_rgb(0.7215686274509804, 0.5254901960784314, 0.043137254901960784);
    pub const DARK_GRAY: Color = Color::create_rgb(0.6627450980392157, 0.6627450980392157, 0.6627450980392157);
    pub const DARK_GREY: Color = Color::create_rgb(0.6627450980392157, 0.6627450980392157, 0.6627450980392157);
    pub const DARK_GREEN: Color = Color::create_rgb(0.0, 0.39215686274509803, 0.0);
    pub const DARK_KHAKI: Color = Color::create_rgb(0.7411764705882353, 0.7176470588235294, 0.4196078431372549);
    pub const DARK_MAGENTA: Color = Color::create_rgb(0.5450980392156862, 0.0, 0.5450980392156862);
    pub const DARK_OLIVE_GREEN: Color = Color::create_rgb(0.3333333333333333, 0.4196078431372549, 0.1843137254901961);
    pub const DARK_ORANGE: Color = Color::create_rgb(1.0, 0.5490196078431373, 0.0);
    pub const DARK_ORCHID: Color = Color::create_rgb(0.6, 0.19607843137254902, 0.8);
    pub const DARK_RED: Color = Color::create_rgb(0.5450980392156862, 0.0, 0.0);
    pub const DARK_SALMON: Color = Color::create_rgb(0.9137254901960784, 0.5882352941176471, 0.47843137254901963);
    pub const DARK_SEA_GREEN: Color = Color::create_rgb(0.5607843137254902, 0.7372549019607844, 0.5607843137254902);
    pub const DARK_SLATE_BLUE: Color = Color::create_rgb(0.2823529411764706, 0.23921568627450981, 0.5450980392156862);
    pub const DARK_SLATE_GRAY: Color = Color::create_rgb(0.1843137254901961, 0.30980392156862746, 0.30980392156862746);
    pub const DARK_SLATE_GREY: Color = Color::create_rgb(0.1843137254901961, 0.30980392156862746, 0.30980392156862746);
    pub const DARK_TURQUOISE: Color = Color::create_rgb(0.0, 0.807843137254902, 0.8196078431372549);
    pub const DARK_VIOLET: Color = Color::create_rgb(0.5803921568627451, 0.0, 0.8274509803921568);
    pub const DEEP_PINK: Color = Color::create_rgb(1.0, 0.0784313725490196, 0.5764705882352941);
    pub const DEEP_SKY_BLUE: Color = Color::create_rgb(0.0, 0.7490196078431373, 1.0);
    pub const DIM_GRAY: Color = Color::create_rgb(0.4117647058823529, 0.4117647058823529, 0.4117647058823529);
    pub const DIM_GREY: Color = Color::create_rgb(0.4117647058823529, 0.4117647058823529, 0.4117647058823529);
    pub const DODGER_BLUE: Color = Color::create_rgb(0.11764705882352941, 0.5647058823529412, 1.0);
    pub const FIRE_BRICK: Color = Color::create_rgb(0.6980392156862745, 0.13333333333333333, 0.13333333333333333);
    pub const FLORAL_WHITE: Color = Color::create_rgb(1.0, 0.9803921568627451, 0.9411764705882353);
    pub const FOREST_GREEN: Color = Color::create_rgb(0.13333333333333333, 0.5450980392156862, 0.13333333333333333);
    pub const FUCHSIA: Color = Color::create_rgb(1.0, 0.0, 1.0);
    pub const GAINSBORO: Color = Color::create_rgb(0.8627450980392157, 0.8627450980392157, 0.8627450980392157);
    pub const GHOST_WHITE: Color = Color::create_rgb(0.9725490196078431, 0.9725490196078431, 1.0);
    pub const GOLD: Color = Color::create_rgb(1.0, 0.8431372549019608, 0.0);
    pub const GOLDEN_ROD: Color = Color::create_rgb(0.8549019607843137, 0.6470588235294118, 0.12549019607843137);
    pub const GRAY: Color = Color::create_rgb(0.5019607843137255, 0.5019607843137255, 0.5019607843137255);
    pub const GREY: Color = Color::create_rgb(0.5019607843137255, 0.5019607843137255, 0.5019607843137255);
    pub const GREEN: Color = Color::create_rgb(0.0, 0.5019607843137255, 0.0);
    pub const GREEN_YELLOW: Color = Color::create_rgb(0.6784313725490196, 1.0, 0.1843137254901961);
    pub const HONEY_DEW: Color = Color::create_rgb(0.9411764705882353, 1.0, 0.9411764705882353);
    pub const HOT_PINK: Color = Color::create_rgb(1.0, 0.4117647058823529, 0.7058823529411765);
    pub const INDIAN_RED: Color = Color::create_rgb(0.803921568627451, 0.3607843137254902, 0.3607843137254902);
    pub const INDIGO: Color = Color::create_rgb(0.29411764705882354, 0.0, 0.5098039215686274);
    pub const IVORY: Color = Color::create_rgb(1.0, 1.0, 0.9411764705882353);
    pub const KHAKI: Color = Color::create_rgb(0.9411764705882353, 0.9019607843137255, 0.5490196078431373);
    pub const LAVENDER: Color = Color::create_rgb(0.9019607843137255, 0.9019607843137255, 0.9803921568627451);
    pub const LAVENDER_BLUSH: Color = Color::create_rgb(1.0, 0.9411764705882353, 0.9607843137254902);
    pub const LAWN_GREEN: Color = Color::create_rgb(0.48627450980392156, 0.9882352941176471, 0.0);
    pub const LEMON_CHIFFON: Color = Color::create_rgb(1.0, 0.9803921568627451, 0.803921568627451);
    pub const LIGHT_BLUE: Color = Color::create_rgb(0.6784313725490196, 0.8470588235294118, 0.9019607843137255);
    pub const LIGHT_CORAL: Color = Color::create_rgb(0.9411764705882353, 0.5019607843137255, 0.5019607843137255);
    pub const LIGHT_CYAN: Color = Color::create_rgb(0.8784313725490196, 1.0, 1.0);
    pub const LIGHT_GOLDEN_ROD_YELLOW: Color = Color::create_rgb(0.9803921568627451, 0.9803921568627451, 0.8235294117647058);
    pub const LIGHT_GRAY: Color = Color::create_rgb(0.8274509803921568, 0.8274509803921568, 0.8274509803921568);
    pub const LIGHT_GREY: Color = Color::create_rgb(0.8274509803921568, 0.8274509803921568, 0.8274509803921568);
    pub const LIGHT_GREEN: Color = Color::create_rgb(0.5647058823529412, 0.9333333333333333, 0.5647058823529412);
    pub const LIGHT_PINK: Color = Color::create_rgb(1.0, 0.7137254901960784, 0.7568627450980392);
    pub const LIGHT_SALMON: Color = Color::create_rgb(1.0, 0.6274509803921569, 0.47843137254901963);
    pub const LIGHT_SEA_GREEN: Color = Color::create_rgb(0.12549019607843137, 0.6980392156862745, 0.6666666666666666);
    pub const LIGHT_SKY_BLUE: Color = Color::create_rgb(0.5294117647058824, 0.807843137254902, 0.9803921568627451);
    pub const LIGHT_SLATE_GRAY: Color = Color::create_rgb(0.4666666666666667, 0.5333333333333333, 0.6);
    pub const LIGHT_SLATE_GREY: Color = Color::create_rgb(0.4666666666666667, 0.5333333333333333, 0.6);
    pub const LIGHT_STEEL_BLUE: Color = Color::create_rgb(0.6901960784313725, 0.7686274509803922, 0.8705882352941177);
    pub const LIGHT_YELLOW: Color = Color::create_rgb(1.0, 1.0, 0.8784313725490196);
    pub const LIME: Color = Color::create_rgb(0.0, 1.0, 0.0);
    pub const LIME_GREEN: Color = Color::create_rgb(0.19607843137254902, 0.803921568627451, 0.19607843137254902);
    pub const LINEN: Color = Color::create_rgb(0.9803921568627451, 0.9411764705882353, 0.9019607843137255);
    pub const MAGENTA: Color = Color::create_rgb(1.0, 0.0, 1.0);
    pub const MAROON: Color = Color::create_rgb(0.5019607843137255, 0.0, 0.0);
    pub const MEDIUM_AQUA_MARINE: Color = Color::create_rgb(0.4, 0.803921568627451, 0.6666666666666666);
    pub const MEDIUM_BLUE: Color = Color::create_rgb(0.0, 0.0, 0.803921568627451);
    pub const MEDIUM_ORCHID: Color = Color::create_rgb(0.7294117647058823, 0.3333333333333333, 0.8274509803921568);
    pub const MEDIUM_PURPLE: Color = Color::create_rgb(0.5764705882352941, 0.4392156862745098, 0.8588235294117647);
    pub const MEDIUM_SEA_GREEN: Color = Color::create_rgb(0.23529411764705882, 0.7019607843137254, 0.44313725490196076);
    pub const MEDIUM_SLATE_BLUE: Color = Color::create_rgb(0.4823529411764706, 0.40784313725490196, 0.9333333333333333);
    pub const MEDIUM_SPRING_GREEN: Color = Color::create_rgb(0.0, 0.9803921568627451, 0.6039215686274509);
    pub const MEDIUM_TURQUOISE: Color = Color::create_rgb(0.2823529411764706, 0.8196078431372549, 0.8);
    pub const MEDIUM_VIOLET_RED: Color = Color::create_rgb(0.7803921568627451, 0.08235294117647059, 0.5215686274509804);
    pub const MIDNIGHT_BLUE: Color = Color::create_rgb(0.09803921568627451, 0.09803921568627451, 0.4392156862745098);
    pub const MINT_CREAM: Color = Color::create_rgb(0.9607843137254902, 1.0, 0.9803921568627451);
    pub const MISTY_ROSE: Color = Color::create_rgb(1.0, 0.8941176470588236, 0.8823529411764706);
    pub const MOCCASIN: Color = Color::create_rgb(1.0, 0.8941176470588236, 0.7098039215686275);
    pub const NAVAJO_WHITE: Color = Color::create_rgb(1.0, 0.8705882352941177, 0.6784313725490196);
    pub const NAVY: Color = Color::create_rgb(0.0, 0.0, 0.5019607843137255);
    pub const OLD_LACE: Color = Color::create_rgb(0.9921568627450981, 0.9607843137254902, 0.9019607843137255);
    pub const OLIVE: Color = Color::create_rgb(0.5019607843137255, 0.5019607843137255, 0.0);
    pub const OLIVE_DRAB: Color = Color::create_rgb(0.4196078431372549, 0.5568627450980392, 0.13725490196078433);
    pub const ORANGE: Color = Color::create_rgb(1.0, 0.6470588235294118, 0.0);
    pub const ORANGE_RED: Color = Color::create_rgb(1.0, 0.27058823529411763, 0.0);
    pub const ORCHID: Color = Color::create_rgb(0.8549019607843137, 0.4392156862745098, 0.8392156862745098);
    pub const PALE_GOLDEN_ROD: Color = Color::create_rgb(0.9333333333333333, 0.9098039215686274, 0.6666666666666666);
    pub const PALE_GREEN: Color = Color::create_rgb(0.596078431372549, 0.984313725490196, 0.596078431372549);
    pub const PALE_TURQUOISE: Color = Color::create_rgb(0.6862745098039216, 0.9333333333333333, 0.9333333333333333);
    pub const PALE_VIOLET_RED: Color = Color::create_rgb(0.8588235294117647, 0.4392156862745098, 0.5764705882352941);
    pub const PAPAYA_WHIP: Color = Color::create_rgb(1.0, 0.9372549019607843, 0.8352941176470589);
    pub const PEACH_PUFF: Color = Color::create_rgb(1.0, 0.8549019607843137, 0.7254901960784313);
    pub const PERU: Color = Color::create_rgb(0.803921568627451, 0.5215686274509804, 0.24705882352941178);
    pub const PINK: Color = Color::create_rgb(1.0, 0.7529411764705882, 0.796078431372549);
    pub const PLUM: Color = Color::create_rgb(0.8666666666666667, 0.6274509803921569, 0.8666666666666667);
    pub const POWDER_BLUE: Color = Color::create_rgb(0.6901960784313725, 0.8784313725490196, 0.9019607843137255);
    pub const PURPLE: Color = Color::create_rgb(0.5019607843137255, 0.0, 0.5019607843137255);
    pub const REBECCA_PURPLE: Color = Color::create_rgb(0.4, 0.2, 0.6);
    pub const RED: Color = Color::create_rgb(1.0, 0.0, 0.0);
    pub const ROSY_BROWN: Color = Color::create_rgb(0.7372549019607844, 0.5607843137254902, 0.5607843137254902);
    pub const ROYAL_BLUE: Color = Color::create_rgb(0.2549019607843137, 0.4117647058823529, 0.8823529411764706);
    pub const SADDLE_BROWN: Color = Color::create_rgb(0.5450980392156862, 0.27058823529411763, 0.07450980392156863);
    pub const SALMON: Color = Color::create_rgb(0.9803921568627451, 0.5019607843137255, 0.4470588235294118);
    pub const SANDY_BROWN: Color = Color::create_rgb(0.9568627450980393, 0.6431372549019608, 0.3764705882352941);
    pub const SEA_GREEN: Color = Color::create_rgb(0.1803921568627451, 0.5450980392156862, 0.3411764705882353);
    pub const SEA_SHELL: Color = Color::create_rgb(1.0, 0.9607843137254902, 0.9333333333333333);
    pub const SIENNA: Color = Color::create_rgb(0.6274509803921569, 0.3215686274509804, 0.17647058823529413);
    pub const SILVER: Color = Color::create_rgb(0.7529411764705882, 0.7529411764705882, 0.7529411764705882);
    pub const SKY_BLUE: Color = Color::create_rgb(0.5294117647058824, 0.807843137254902, 0.9215686274509803);
    pub const SLATE_BLUE: Color = Color::create_rgb(0.41568627450980394, 0.35294117647058826, 0.803921568627451);
    pub const SLATE_GRAY: Color = Color::create_rgb(0.4392156862745098, 0.5019607843137255, 0.5647058823529412);
    pub const SLATE_GREY: Color = Color::create_rgb(0.4392156862745098, 0.5019607843137255, 0.5647058823529412);
    pub const SNOW: Color = Color::create_rgb(1.0, 0.9803921568627451, 0.9803921568627451);
    pub const SPRING_GREEN: Color = Color::create_rgb(0.0, 1.0, 0.4980392156862745);
    pub const STEEL_BLUE: Color = Color::create_rgb(0.27450980392156865, 0.5098039215686274, 0.7058823529411765);
    pub const TAN: Color = Color::create_rgb(0.8235294117647058, 0.7058823529411765, 0.5490196078431373);
    pub const TEAL: Color = Color::create_rgb(0.0, 0.5019607843137255, 0.5019607843137255);
    pub const THISTLE: Color = Color::create_rgb(0.8470588235294118, 0.7490196078431373, 0.8470588235294118);
    pub const TOMATO: Color = Color::create_rgb(1.0, 0.38823529411764707, 0.2784313725490196);
    pub const TURQUOISE: Color = Color::create_rgb(0.25098039215686274, 0.8784313725490196, 0.8156862745098039);
    pub const VIOLET: Color = Color::create_rgb(0.9333333333333333, 0.5098039215686274, 0.9333333333333333);
    pub const WHEAT: Color = Color::create_rgb(0.9607843137254902, 0.8705882352941177, 0.7019607843137254);
    pub const WHITE: Color = Color::create_rgb(1.0, 1.0, 1.0);
    pub const WHITE_SMOKE: Color = Color::create_rgb(0.9607843137254902, 0.9607843137254902, 0.9607843137254902);
    pub const YELLOW: Color = Color::create_rgb(1.0, 1.0, 0.0);
    pub const YELLOW_GREEN: Color = Color::create_rgb(0.6039215686274509, 0.803921568627451, 0.19607843137254902);

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
