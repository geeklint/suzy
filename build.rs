extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn tui() {
    // Tell cargo to tell rustc to link the system ncurses
    // shared library.
    println!("cargo:rustc-link-lib=ncurses");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=include/tui.h");

    let bindings = bindgen::Builder::default()
        .header("include/tui.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate tui bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let path = out_path.join("tui_bindings.rs");
    let debug = path.to_string_lossy().to_string();
    bindings
        .write_to_file(path)
        .expect(&format!("Couldn't write {}!", debug));
}

fn main() {
    if cfg!(feature="tui") { tui() };
}
