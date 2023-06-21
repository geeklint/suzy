use std::path::Path;

use suzy_build_tools::fontasset::{latin1, FontAtlas, TextureDim::V1024};

fn main() {
    let src_dir = Path::new(file!()).parent().unwrap();
    let roboto_dir = src_dir.join("roboto");

    let workspace_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let dest = workspace_dir
        .join("suzy-default-font")
        .join("src")
        .join("default_font.rs");

    FontAtlas::with_texture_size(V1024, V1024)
        .with_padding_ratio(0.2)
        .add_font(
            "regular".to_string(),
            roboto_dir.join("Roboto-Regular.ttf"),
            latin1(),
        )
        .unwrap()
        .add_font(
            "bold".to_string(),
            roboto_dir.join("Roboto-Bold.ttf"),
            latin1(),
        )
        .unwrap()
        .write_module(dest)
        .unwrap();
}
