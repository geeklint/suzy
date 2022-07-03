use suzy_build_tools::fontasset::{self, AssetSize, FontFamily, Settings};

fn main() {
    let mut target = String::from("font.rs");
    let mut font_path = String::new();
    let mut bold_path = None;
    let mut italic_path = None;
    let mut bold_italic_path = None;
    let mut font_size = String::from("24");
    let mut next_arg = &mut font_path;
    for arg in std::env::args() {
        if let Some(cmd) = arg.strip_prefix("--") {
            next_arg = match cmd {
                "output" => &mut target,
                "font" => &mut font_path,
                "bold" => bold_path.get_or_insert(String::new()),
                "italic" => italic_path.get_or_insert(String::new()),
                "bold-italic" => bold_italic_path.get_or_insert(String::new()),
                "font-size" => &mut font_size,
                _ => panic!("unexpected argument {}", arg),
            }
        } else {
            *next_arg = arg;
            next_arg = &mut font_path;
        }
    }
    let mut family = FontFamily::font_path(&font_path);
    if let Some(bold_path) = &bold_path {
        family = family.bold_path(bold_path);
    }
    if let Some(italic_path) = &italic_path {
        family = family.italic_path(italic_path);
    }
    if let Some(bold_italic_path) = &bold_italic_path {
        family = family.bold_italic_path(bold_italic_path);
    }
    let font_size = font_size.parse().unwrap();
    let settings = Settings::new()
        .show_progress()
        .latin1()
        .size(AssetSize::FontSize(font_size));
    fontasset::build_fontasset(family, target, settings);
}
