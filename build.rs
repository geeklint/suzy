
use std::env;
use std::path::Path;
use std::fs::OpenOptions;

#[cfg(feature = "tui")]
fn tui(out_dir: impl AsRef<Path>) {
    // Tell cargo to tell rustc to link the system ncurses
    // shared library.
    println!("cargo:rustc-link-lib=ncursesw");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=include/tui.h");

    let bindings = bindgen::Builder::default()
        .header("include/tui.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate tui bindings");

    let path = out_dir.as_ref().join("tui_bindings.rs");
    let debug = path.to_string_lossy();
    bindings
        .write_to_file(path)
        .expect(&format!("Couldn't write {}!", &debug));
}

fn main() {
    let out_dir = env::var("OUT_DIR").expect("expected cargo to set OUT_DIR");

    #[cfg(feature="tui")] { tui(&out_dir) };
}
