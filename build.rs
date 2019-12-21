extern crate bindgen;
extern crate shlex;

use std::env;
use std::path::PathBuf;
use std::process::Command;

use bindgen::callbacks::{
    ParseCallbacks,
    EnumVariantValue,
    EnumVariantCustomBehavior,
};

fn tui() {
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

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let path = out_path.join("tui_bindings.rs");
    let debug = path.to_string_lossy().to_string();
    bindings
        .write_to_file(path)
        .expect(&format!("Couldn't write {}!", debug));
}

fn sdl2_config(arg: &str) -> String {
    let config_output = Command::new("sdl2-config")
        .args(&[arg])
        .output()
        .expect("failed to execute sdl2-config");
    assert!(
        config_output.status.success(),
        "sdl2-config returned error code",
    );
    String::from_utf8(config_output.stdout)
        .expect("sdl2-config output wasn't valid utf-8")
}

#[derive(Copy, Clone, Debug)]
struct BlacklistMath;

impl ParseCallbacks for BlacklistMath {
    fn enum_variant_behavior(
        &self, 
        _enum_name: Option<&str>, 
        _variant_name: &str, 
        _variant_value: EnumVariantValue
    ) -> Option<EnumVariantCustomBehavior> {
        eprintln!("variant_name: {}", _variant_name);
        match _variant_name {
            "FP_NAN"
            | "FP_INFINITE"
            | "FP_ZERO"
            | "FP_SUBNORMAL"
            | "FP_NORMAL" => Some(EnumVariantCustomBehavior::Hide),
            _ => None,
        }
    }
}

fn sdl() {
    let cflags = shlex::split(&sdl2_config("--cflags"))
        .expect("Unable to parse sdl2-config output");
    let libs = sdl2_config("--libs");

    println!("cargo:rustc-flags={}", libs);
    println!("cargo:rerun-if-changed=include/sdl.h");

    let bindings = bindgen::Builder::default()
        .clang_args(cflags)
        .header("include/sdl.h")
        .parse_callbacks(Box::new(BlacklistMath))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate sdl2 bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let path = out_path.join("sdl_bindings.rs");
    let debug = path.to_string_lossy().to_string();
    bindings
        .write_to_file(path)
        .expect(&format!("Couldn't write {}!", debug));
}

fn main() {
    if cfg!(feature="tui") { tui() };
    if cfg!(feature="sdl") { sdl() };
}
