use suzy_fontasset_builder::*;

fn main() {
    let chars = (0..0x7f)
        .map(|i| i.into())
        .filter(|ch: &char| ch.is_ascii_graphic())
        .collect::<Vec<_>>();
    let settings = Settings {
        chars,
        target_size: 72,
        max_distance: 20,
    };
    build_fontasset(
        "/usr/lib/python2.7/dist-packages/kivy/data/fonts/Roboto-Regular.ttf",
        "final_atlas.rs",
        settings,
    );
}
