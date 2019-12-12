use super::CubicBezier;
use super::Color;

pub const EASE_LINEAR: CubicBezier = CubicBezier(0.0, 0.0, 1.0, 1.0);
pub const EASE_IN_SINE: CubicBezier = CubicBezier(0.47, 0.0, 0.745, 0.715);
pub const EASE_OUT_SINE: CubicBezier = CubicBezier(0.39, 0.575, 0.565, 1.0);
pub const EASE_IN_OUT_SINE: CubicBezier = CubicBezier(0.445, 0.05, 0.55, 0.95);
pub const EASE_IN_QUAD: CubicBezier = CubicBezier(0.55, 0.085, 0.68, 0.53);
pub const EASE_OUT_QUAD: CubicBezier = CubicBezier(0.25, 0.46, 0.45, 0.94);
pub const EASE_IN_OUT_QUAD: CubicBezier = CubicBezier(0.455, 0.03, 0.515, 0.955);
pub const EASE_IN_CUBIC: CubicBezier = CubicBezier(0.55, 0.055, 0.675, 0.19);
pub const EASE_OUT_CUBIC: CubicBezier = CubicBezier(0.215, 0.61, 0.355, 1.0);
pub const EASE_IN_OUT_CUBIC: CubicBezier = CubicBezier(0.645, 0.045, 0.355, 1.0);
pub const EASE_IN_QUART: CubicBezier = CubicBezier(0.895, 0.03, 0.685, 0.22);
pub const EASE_OUT_QUART: CubicBezier = CubicBezier(0.165, 0.84, 0.44, 1.0);
pub const EASE_IN_OUT_QUART: CubicBezier = CubicBezier(0.77, 0.0, 0.175, 1.0);
pub const EASE_IN_QUINT: CubicBezier = CubicBezier(0.755, 0.05, 0.855, 0.06);
pub const EASE_OUT_QUINT: CubicBezier = CubicBezier(0.23, 1.0, 0.32, 1.0);
pub const EASE_IN_OUT_QUINT: CubicBezier = CubicBezier(0.86, 0.0, 0.07, 1.0);
pub const EASE_IN_EXPO: CubicBezier = CubicBezier(0.95, 0.05, 0.795, 0.035);
pub const EASE_OUT_EXPO: CubicBezier = CubicBezier(0.19, 1.0, 0.22, 1.0);
pub const EASE_IN_OUT_EXPO: CubicBezier = CubicBezier(1.0, 0.0, 0.0, 1.0);
pub const EASE_IN_CIRC: CubicBezier = CubicBezier(0.6, 0.04, 0.98, 0.335);
pub const EASE_OUT_CIRC: CubicBezier = CubicBezier(0.075, 0.82, 0.165, 1.0);
pub const EASE_IN_OUT_CIRC: CubicBezier = CubicBezier(0.785, 0.135, 0.15, 0.86);
pub const EASE_IN_BACK: CubicBezier = CubicBezier(0.6, -0.28, 0.735, 0.045);
pub const EASE_OUT_BACK: CubicBezier = CubicBezier(0.175, 0.885, 0.32, 1.275);
pub const EASE_IN_OUT_BACK: CubicBezier = CubicBezier(0.68, -0.55, 0.265, 1.55);

pub const ALICE_BLUE: Color = Color::rgb(0.9411764705882353, 0.9725490196078431, 1.0);
pub const ANTIQUE_WHITE: Color = Color::rgb(0.9803921568627451, 0.9215686274509803, 0.8431372549019608);
pub const AQUA: Color = Color::rgb(0.0, 1.0, 1.0);
pub const AQUAMARINE: Color = Color::rgb(0.4980392156862745, 1.0, 0.8313725490196079);
pub const AZURE: Color = Color::rgb(0.9411764705882353, 1.0, 1.0);
pub const BEIGE: Color = Color::rgb(0.9607843137254902, 0.9607843137254902, 0.8627450980392157);
pub const BISQUE: Color = Color::rgb(1.0, 0.8941176470588236, 0.7686274509803922);
pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
pub const BLANCHED_ALMOND: Color = Color::rgb(1.0, 0.9215686274509803, 0.803921568627451);
pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
pub const BLUE_VIOLET: Color = Color::rgb(0.5411764705882353, 0.16862745098039217, 0.8862745098039215);
pub const BROWN: Color = Color::rgb(0.6470588235294118, 0.16470588235294117, 0.16470588235294117);
pub const BURLY_WOOD: Color = Color::rgb(0.8705882352941177, 0.7215686274509804, 0.5294117647058824);
pub const CADET_BLUE: Color = Color::rgb(0.37254901960784315, 0.6196078431372549, 0.6274509803921569);
pub const CHARTREUSE: Color = Color::rgb(0.4980392156862745, 1.0, 0.0);
pub const CHOCOLATE: Color = Color::rgb(0.8235294117647058, 0.4117647058823529, 0.11764705882352941);
pub const CORAL: Color = Color::rgb(1.0, 0.4980392156862745, 0.3137254901960784);
pub const CORNFLOWER_BLUE: Color = Color::rgb(0.39215686274509803, 0.5843137254901961, 0.9294117647058824);
pub const CORNSILK: Color = Color::rgb(1.0, 0.9725490196078431, 0.8627450980392157);
pub const CRIMSON: Color = Color::rgb(0.8627450980392157, 0.0784313725490196, 0.23529411764705882);
pub const CYAN: Color = Color::rgb(0.0, 1.0, 1.0);
pub const DARK_BLUE: Color = Color::rgb(0.0, 0.0, 0.5450980392156862);
pub const DARK_CYAN: Color = Color::rgb(0.0, 0.5450980392156862, 0.5450980392156862);
pub const DARK_GOLDEN_ROD: Color = Color::rgb(0.7215686274509804, 0.5254901960784314, 0.043137254901960784);
pub const DARK_GRAY: Color = Color::rgb(0.6627450980392157, 0.6627450980392157, 0.6627450980392157);
pub const DARK_GREY: Color = Color::rgb(0.6627450980392157, 0.6627450980392157, 0.6627450980392157);
pub const DARK_GREEN: Color = Color::rgb(0.0, 0.39215686274509803, 0.0);
pub const DARK_KHAKI: Color = Color::rgb(0.7411764705882353, 0.7176470588235294, 0.4196078431372549);
pub const DARK_MAGENTA: Color = Color::rgb(0.5450980392156862, 0.0, 0.5450980392156862);
pub const DARK_OLIVE_GREEN: Color = Color::rgb(0.3333333333333333, 0.4196078431372549, 0.1843137254901961);
pub const DARK_ORANGE: Color = Color::rgb(1.0, 0.5490196078431373, 0.0);
pub const DARK_ORCHID: Color = Color::rgb(0.6, 0.19607843137254902, 0.8);
pub const DARK_RED: Color = Color::rgb(0.5450980392156862, 0.0, 0.0);
pub const DARK_SALMON: Color = Color::rgb(0.9137254901960784, 0.5882352941176471, 0.47843137254901963);
pub const DARK_SEA_GREEN: Color = Color::rgb(0.5607843137254902, 0.7372549019607844, 0.5607843137254902);
pub const DARK_SLATE_BLUE: Color = Color::rgb(0.2823529411764706, 0.23921568627450981, 0.5450980392156862);
pub const DARK_SLATE_GRAY: Color = Color::rgb(0.1843137254901961, 0.30980392156862746, 0.30980392156862746);
pub const DARK_SLATE_GREY: Color = Color::rgb(0.1843137254901961, 0.30980392156862746, 0.30980392156862746);
pub const DARK_TURQUOISE: Color = Color::rgb(0.0, 0.807843137254902, 0.8196078431372549);
pub const DARK_VIOLET: Color = Color::rgb(0.5803921568627451, 0.0, 0.8274509803921568);
pub const DEEP_PINK: Color = Color::rgb(1.0, 0.0784313725490196, 0.5764705882352941);
pub const DEEP_SKY_BLUE: Color = Color::rgb(0.0, 0.7490196078431373, 1.0);
pub const DIM_GRAY: Color = Color::rgb(0.4117647058823529, 0.4117647058823529, 0.4117647058823529);
pub const DIM_GREY: Color = Color::rgb(0.4117647058823529, 0.4117647058823529, 0.4117647058823529);
pub const DODGER_BLUE: Color = Color::rgb(0.11764705882352941, 0.5647058823529412, 1.0);
pub const FIRE_BRICK: Color = Color::rgb(0.6980392156862745, 0.13333333333333333, 0.13333333333333333);
pub const FLORAL_WHITE: Color = Color::rgb(1.0, 0.9803921568627451, 0.9411764705882353);
pub const FOREST_GREEN: Color = Color::rgb(0.13333333333333333, 0.5450980392156862, 0.13333333333333333);
pub const FUCHSIA: Color = Color::rgb(1.0, 0.0, 1.0);
pub const GAINSBORO: Color = Color::rgb(0.8627450980392157, 0.8627450980392157, 0.8627450980392157);
pub const GHOST_WHITE: Color = Color::rgb(0.9725490196078431, 0.9725490196078431, 1.0);
pub const GOLD: Color = Color::rgb(1.0, 0.8431372549019608, 0.0);
pub const GOLDEN_ROD: Color = Color::rgb(0.8549019607843137, 0.6470588235294118, 0.12549019607843137);
pub const GRAY: Color = Color::rgb(0.5019607843137255, 0.5019607843137255, 0.5019607843137255);
pub const GREY: Color = Color::rgb(0.5019607843137255, 0.5019607843137255, 0.5019607843137255);
pub const GREEN: Color = Color::rgb(0.0, 0.5019607843137255, 0.0);
pub const GREEN_YELLOW: Color = Color::rgb(0.6784313725490196, 1.0, 0.1843137254901961);
pub const HONEY_DEW: Color = Color::rgb(0.9411764705882353, 1.0, 0.9411764705882353);
pub const HOT_PINK: Color = Color::rgb(1.0, 0.4117647058823529, 0.7058823529411765);
pub const INDIAN_RED: Color = Color::rgb(0.803921568627451, 0.3607843137254902, 0.3607843137254902);
pub const INDIGO: Color = Color::rgb(0.29411764705882354, 0.0, 0.5098039215686274);
pub const IVORY: Color = Color::rgb(1.0, 1.0, 0.9411764705882353);
pub const KHAKI: Color = Color::rgb(0.9411764705882353, 0.9019607843137255, 0.5490196078431373);
pub const LAVENDER: Color = Color::rgb(0.9019607843137255, 0.9019607843137255, 0.9803921568627451);
pub const LAVENDER_BLUSH: Color = Color::rgb(1.0, 0.9411764705882353, 0.9607843137254902);
pub const LAWN_GREEN: Color = Color::rgb(0.48627450980392156, 0.9882352941176471, 0.0);
pub const LEMON_CHIFFON: Color = Color::rgb(1.0, 0.9803921568627451, 0.803921568627451);
pub const LIGHT_BLUE: Color = Color::rgb(0.6784313725490196, 0.8470588235294118, 0.9019607843137255);
pub const LIGHT_CORAL: Color = Color::rgb(0.9411764705882353, 0.5019607843137255, 0.5019607843137255);
pub const LIGHT_CYAN: Color = Color::rgb(0.8784313725490196, 1.0, 1.0);
pub const LIGHT_GOLDEN_ROD_YELLOW: Color = Color::rgb(0.9803921568627451, 0.9803921568627451, 0.8235294117647058);
pub const LIGHT_GRAY: Color = Color::rgb(0.8274509803921568, 0.8274509803921568, 0.8274509803921568);
pub const LIGHT_GREY: Color = Color::rgb(0.8274509803921568, 0.8274509803921568, 0.8274509803921568);
pub const LIGHT_GREEN: Color = Color::rgb(0.5647058823529412, 0.9333333333333333, 0.5647058823529412);
pub const LIGHT_PINK: Color = Color::rgb(1.0, 0.7137254901960784, 0.7568627450980392);
pub const LIGHT_SALMON: Color = Color::rgb(1.0, 0.6274509803921569, 0.47843137254901963);
pub const LIGHT_SEA_GREEN: Color = Color::rgb(0.12549019607843137, 0.6980392156862745, 0.6666666666666666);
pub const LIGHT_SKY_BLUE: Color = Color::rgb(0.5294117647058824, 0.807843137254902, 0.9803921568627451);
pub const LIGHT_SLATE_GRAY: Color = Color::rgb(0.4666666666666667, 0.5333333333333333, 0.6);
pub const LIGHT_SLATE_GREY: Color = Color::rgb(0.4666666666666667, 0.5333333333333333, 0.6);
pub const LIGHT_STEEL_BLUE: Color = Color::rgb(0.6901960784313725, 0.7686274509803922, 0.8705882352941177);
pub const LIGHT_YELLOW: Color = Color::rgb(1.0, 1.0, 0.8784313725490196);
pub const LIME: Color = Color::rgb(0.0, 1.0, 0.0);
pub const LIME_GREEN: Color = Color::rgb(0.19607843137254902, 0.803921568627451, 0.19607843137254902);
pub const LINEN: Color = Color::rgb(0.9803921568627451, 0.9411764705882353, 0.9019607843137255);
pub const MAGENTA: Color = Color::rgb(1.0, 0.0, 1.0);
pub const MAROON: Color = Color::rgb(0.5019607843137255, 0.0, 0.0);
pub const MEDIUM_AQUA_MARINE: Color = Color::rgb(0.4, 0.803921568627451, 0.6666666666666666);
pub const MEDIUM_BLUE: Color = Color::rgb(0.0, 0.0, 0.803921568627451);
pub const MEDIUM_ORCHID: Color = Color::rgb(0.7294117647058823, 0.3333333333333333, 0.8274509803921568);
pub const MEDIUM_PURPLE: Color = Color::rgb(0.5764705882352941, 0.4392156862745098, 0.8588235294117647);
pub const MEDIUM_SEA_GREEN: Color = Color::rgb(0.23529411764705882, 0.7019607843137254, 0.44313725490196076);
pub const MEDIUM_SLATE_BLUE: Color = Color::rgb(0.4823529411764706, 0.40784313725490196, 0.9333333333333333);
pub const MEDIUM_SPRING_GREEN: Color = Color::rgb(0.0, 0.9803921568627451, 0.6039215686274509);
pub const MEDIUM_TURQUOISE: Color = Color::rgb(0.2823529411764706, 0.8196078431372549, 0.8);
pub const MEDIUM_VIOLET_RED: Color = Color::rgb(0.7803921568627451, 0.08235294117647059, 0.5215686274509804);
pub const MIDNIGHT_BLUE: Color = Color::rgb(0.09803921568627451, 0.09803921568627451, 0.4392156862745098);
pub const MINT_CREAM: Color = Color::rgb(0.9607843137254902, 1.0, 0.9803921568627451);
pub const MISTY_ROSE: Color = Color::rgb(1.0, 0.8941176470588236, 0.8823529411764706);
pub const MOCCASIN: Color = Color::rgb(1.0, 0.8941176470588236, 0.7098039215686275);
pub const NAVAJO_WHITE: Color = Color::rgb(1.0, 0.8705882352941177, 0.6784313725490196);
pub const NAVY: Color = Color::rgb(0.0, 0.0, 0.5019607843137255);
pub const OLD_LACE: Color = Color::rgb(0.9921568627450981, 0.9607843137254902, 0.9019607843137255);
pub const OLIVE: Color = Color::rgb(0.5019607843137255, 0.5019607843137255, 0.0);
pub const OLIVE_DRAB: Color = Color::rgb(0.4196078431372549, 0.5568627450980392, 0.13725490196078433);
pub const ORANGE: Color = Color::rgb(1.0, 0.6470588235294118, 0.0);
pub const ORANGE_RED: Color = Color::rgb(1.0, 0.27058823529411763, 0.0);
pub const ORCHID: Color = Color::rgb(0.8549019607843137, 0.4392156862745098, 0.8392156862745098);
pub const PALE_GOLDEN_ROD: Color = Color::rgb(0.9333333333333333, 0.9098039215686274, 0.6666666666666666);
pub const PALE_GREEN: Color = Color::rgb(0.596078431372549, 0.984313725490196, 0.596078431372549);
pub const PALE_TURQUOISE: Color = Color::rgb(0.6862745098039216, 0.9333333333333333, 0.9333333333333333);
pub const PALE_VIOLET_RED: Color = Color::rgb(0.8588235294117647, 0.4392156862745098, 0.5764705882352941);
pub const PAPAYA_WHIP: Color = Color::rgb(1.0, 0.9372549019607843, 0.8352941176470589);
pub const PEACH_PUFF: Color = Color::rgb(1.0, 0.8549019607843137, 0.7254901960784313);
pub const PERU: Color = Color::rgb(0.803921568627451, 0.5215686274509804, 0.24705882352941178);
pub const PINK: Color = Color::rgb(1.0, 0.7529411764705882, 0.796078431372549);
pub const PLUM: Color = Color::rgb(0.8666666666666667, 0.6274509803921569, 0.8666666666666667);
pub const POWDER_BLUE: Color = Color::rgb(0.6901960784313725, 0.8784313725490196, 0.9019607843137255);
pub const PURPLE: Color = Color::rgb(0.5019607843137255, 0.0, 0.5019607843137255);
pub const REBECCA_PURPLE: Color = Color::rgb(0.4, 0.2, 0.6);
pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
pub const ROSY_BROWN: Color = Color::rgb(0.7372549019607844, 0.5607843137254902, 0.5607843137254902);
pub const ROYAL_BLUE: Color = Color::rgb(0.2549019607843137, 0.4117647058823529, 0.8823529411764706);
pub const SADDLE_BROWN: Color = Color::rgb(0.5450980392156862, 0.27058823529411763, 0.07450980392156863);
pub const SALMON: Color = Color::rgb(0.9803921568627451, 0.5019607843137255, 0.4470588235294118);
pub const SANDY_BROWN: Color = Color::rgb(0.9568627450980393, 0.6431372549019608, 0.3764705882352941);
pub const SEA_GREEN: Color = Color::rgb(0.1803921568627451, 0.5450980392156862, 0.3411764705882353);
pub const SEA_SHELL: Color = Color::rgb(1.0, 0.9607843137254902, 0.9333333333333333);
pub const SIENNA: Color = Color::rgb(0.6274509803921569, 0.3215686274509804, 0.17647058823529413);
pub const SILVER: Color = Color::rgb(0.7529411764705882, 0.7529411764705882, 0.7529411764705882);
pub const SKY_BLUE: Color = Color::rgb(0.5294117647058824, 0.807843137254902, 0.9215686274509803);
pub const SLATE_BLUE: Color = Color::rgb(0.41568627450980394, 0.35294117647058826, 0.803921568627451);
pub const SLATE_GRAY: Color = Color::rgb(0.4392156862745098, 0.5019607843137255, 0.5647058823529412);
pub const SLATE_GREY: Color = Color::rgb(0.4392156862745098, 0.5019607843137255, 0.5647058823529412);
pub const SNOW: Color = Color::rgb(1.0, 0.9803921568627451, 0.9803921568627451);
pub const SPRING_GREEN: Color = Color::rgb(0.0, 1.0, 0.4980392156862745);
pub const STEEL_BLUE: Color = Color::rgb(0.27450980392156865, 0.5098039215686274, 0.7058823529411765);
pub const TAN: Color = Color::rgb(0.8235294117647058, 0.7058823529411765, 0.5490196078431373);
pub const TEAL: Color = Color::rgb(0.0, 0.5019607843137255, 0.5019607843137255);
pub const THISTLE: Color = Color::rgb(0.8470588235294118, 0.7490196078431373, 0.8470588235294118);
pub const TOMATO: Color = Color::rgb(1.0, 0.38823529411764707, 0.2784313725490196);
pub const TURQUOISE: Color = Color::rgb(0.25098039215686274, 0.8784313725490196, 0.8156862745098039);
pub const VIOLET: Color = Color::rgb(0.9333333333333333, 0.5098039215686274, 0.9333333333333333);
pub const WHEAT: Color = Color::rgb(0.9607843137254902, 0.8705882352941177, 0.7019607843137254);
pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
pub const WHITE_SMOKE: Color = Color::rgb(0.9607843137254902, 0.9607843137254902, 0.9607843137254902);
pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
pub const YELLOW_GREEN: Color = Color::rgb(0.6039215686274509, 0.803921568627451, 0.19607843137254902);

#[cfg(feature = "lookup_consts")]
use lazy_static::lazy_static;
#[cfg(feature = "lookup_consts")]
use std::collections::HashMap;

#[cfg(feature = "lookup_consts")]
lazy_static!{
    pub static ref COLOR_NAMES: HashMap<&'static str, Color> = [
        ("aliceblue", ALICE_BLUE),
        ("antiquewhite", ANTIQUE_WHITE),
        ("aqua", AQUA),
        ("aquamarine", AQUAMARINE),
        ("azure", AZURE),
        ("beige", BEIGE),
        ("bisque", BISQUE),
        ("black", BLACK),
        ("blanchedalmond", BLANCHED_ALMOND),
        ("blue", BLUE),
        ("blueviolet", BLUE_VIOLET),
        ("brown", BROWN),
        ("burlywood", BURLY_WOOD),
        ("cadetblue", CADET_BLUE),
        ("chartreuse", CHARTREUSE),
        ("chocolate", CHOCOLATE),
        ("coral", CORAL),
        ("cornflowerblue", CORNFLOWER_BLUE),
        ("cornsilk", CORNSILK),
        ("crimson", CRIMSON),
        ("cyan", CYAN),
        ("darkblue", DARK_BLUE),
        ("darkcyan", DARK_CYAN),
        ("darkgoldenrod", DARK_GOLDEN_ROD),
        ("darkgray", DARK_GRAY),
        ("darkgrey", DARK_GREY),
        ("darkgreen", DARK_GREEN),
        ("darkkhaki", DARK_KHAKI),
        ("darkmagenta", DARK_MAGENTA),
        ("darkolivegreen", DARK_OLIVE_GREEN),
        ("darkorange", DARK_ORANGE),
        ("darkorchid", DARK_ORCHID),
        ("darkred", DARK_RED),
        ("darksalmon", DARK_SALMON),
        ("darkseagreen", DARK_SEA_GREEN),
        ("darkslateblue", DARK_SLATE_BLUE),
        ("darkslategray", DARK_SLATE_GRAY),
        ("darkslategrey", DARK_SLATE_GREY),
        ("darkturquoise", DARK_TURQUOISE),
        ("darkviolet", DARK_VIOLET),
        ("deeppink", DEEP_PINK),
        ("deepskyblue", DEEP_SKY_BLUE),
        ("dimgray", DIM_GRAY),
        ("dimgrey", DIM_GREY),
        ("dodgerblue", DODGER_BLUE),
        ("firebrick", FIRE_BRICK),
        ("floralwhite", FLORAL_WHITE),
        ("forestgreen", FOREST_GREEN),
        ("fuchsia", FUCHSIA),
        ("gainsboro", GAINSBORO),
        ("ghostwhite", GHOST_WHITE),
        ("gold", GOLD),
        ("goldenrod", GOLDEN_ROD),
        ("gray", GRAY),
        ("grey", GREY),
        ("green", GREEN),
        ("greenyellow", GREEN_YELLOW),
        ("honeydew", HONEY_DEW),
        ("hotpink", HOT_PINK),
        ("indianred", INDIAN_RED),
        ("indigo", INDIGO),
        ("ivory", IVORY),
        ("khaki", KHAKI),
        ("lavender", LAVENDER),
        ("lavenderblush", LAVENDER_BLUSH),
        ("lawngreen", LAWN_GREEN),
        ("lemonchiffon", LEMON_CHIFFON),
        ("lightblue", LIGHT_BLUE),
        ("lightcoral", LIGHT_CORAL),
        ("lightcyan", LIGHT_CYAN),
        ("lightgoldenrodyellow", LIGHT_GOLDEN_ROD_YELLOW),
        ("lightgray", LIGHT_GRAY),
        ("lightgrey", LIGHT_GREY),
        ("lightgreen", LIGHT_GREEN),
        ("lightpink", LIGHT_PINK),
        ("lightsalmon", LIGHT_SALMON),
        ("lightseagreen", LIGHT_SEA_GREEN),
        ("lightskyblue", LIGHT_SKY_BLUE),
        ("lightslategray", LIGHT_SLATE_GRAY),
        ("lightslategrey", LIGHT_SLATE_GREY),
        ("lightsteelblue", LIGHT_STEEL_BLUE),
        ("lightyellow", LIGHT_YELLOW),
        ("lime", LIME),
        ("limegreen", LIME_GREEN),
        ("linen", LINEN),
        ("magenta", MAGENTA),
        ("maroon", MAROON),
        ("mediumaquamarine", MEDIUM_AQUA_MARINE),
        ("mediumblue", MEDIUM_BLUE),
        ("mediumorchid", MEDIUM_ORCHID),
        ("mediumpurple", MEDIUM_PURPLE),
        ("mediumseagreen", MEDIUM_SEA_GREEN),
        ("mediumslateblue", MEDIUM_SLATE_BLUE),
        ("mediumspringgreen", MEDIUM_SPRING_GREEN),
        ("mediumturquoise", MEDIUM_TURQUOISE),
        ("mediumvioletred", MEDIUM_VIOLET_RED),
        ("midnightblue", MIDNIGHT_BLUE),
        ("mintcream", MINT_CREAM),
        ("mistyrose", MISTY_ROSE),
        ("moccasin", MOCCASIN),
        ("navajowhite", NAVAJO_WHITE),
        ("navy", NAVY),
        ("oldlace", OLD_LACE),
        ("olive", OLIVE),
        ("olivedrab", OLIVE_DRAB),
        ("orange", ORANGE),
        ("orangered", ORANGE_RED),
        ("orchid", ORCHID),
        ("palegoldenrod", PALE_GOLDEN_ROD),
        ("palegreen", PALE_GREEN),
        ("paleturquoise", PALE_TURQUOISE),
        ("palevioletred", PALE_VIOLET_RED),
        ("papayawhip", PAPAYA_WHIP),
        ("peachpuff", PEACH_PUFF),
        ("peru", PERU),
        ("pink", PINK),
        ("plum", PLUM),
        ("powderblue", POWDER_BLUE),
        ("purple", PURPLE),
        ("rebeccapurple", REBECCA_PURPLE),
        ("red", RED),
        ("rosybrown", ROSY_BROWN),
        ("royalblue", ROYAL_BLUE),
        ("saddlebrown", SADDLE_BROWN),
        ("salmon", SALMON),
        ("sandybrown", SANDY_BROWN),
        ("seagreen", SEA_GREEN),
        ("seashell", SEA_SHELL),
        ("sienna", SIENNA),
        ("silver", SILVER),
        ("skyblue", SKY_BLUE),
        ("slateblue", SLATE_BLUE),
        ("slategray", SLATE_GRAY),
        ("slategrey", SLATE_GREY),
        ("snow", SNOW),
        ("springgreen", SPRING_GREEN),
        ("steelblue", STEEL_BLUE),
        ("tan", TAN),
        ("teal", TEAL),
        ("thistle", THISTLE),
        ("tomato", TOMATO),
        ("turquoise", TURQUOISE),
        ("violet", VIOLET),
        ("wheat", WHEAT),
        ("white", WHITE),
        ("whitesmoke", WHITE_SMOKE),
        ("yellow", YELLOW),
        ("yellowgreen", YELLOW_GREEN),
    ].iter().copied().collect();
}
