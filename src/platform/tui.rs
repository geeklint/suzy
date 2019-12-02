#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/tui_bindings.rs"));

use std::env;

pub struct Window {
}

impl Window {
    pub fn new() -> Result<Self, ()> {
        let term = env::var_os("TERM");
    }
}

pub enum Texture {
    Uniform(char),
    
}
