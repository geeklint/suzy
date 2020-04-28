
use std::env;
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};

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

#[cfg(feature = "opengl")]
fn opengl(out_dir: impl AsRef<Path>) {
    use gl_generator::{Registry, Api, Profile, Fallbacks, StructGenerator};

    let path = out_dir.as_ref().join("opengl_bindings.rs");
    let debug = path.to_string_lossy();
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .expect(&format!("Couldn't open {}!", &debug));

    Registry::new(Api::Gles2, (3, 0), Profile::Core, Fallbacks::All, [])
        .write_bindings(StructGenerator, &mut file)
        .expect(&format!("Couldn't write {}!", &debug));
}

fn main() {
    let out_dir = env::var("OUT_DIR").expect("expected cargo to set OUT_DIR");

    #[cfg(feature="tui")] { tui(&out_dir) };
    #[cfg(feature="opengl")] { opengl(&out_dir) };
}
