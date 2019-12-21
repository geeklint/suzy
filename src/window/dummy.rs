use std::io;

pub struct Window {}

impl Window {
    pub fn new() -> Result<Self, io::Error> {
        unimplemented!()
    }

    pub fn get_size(&self) -> (f32, f32) {
        unimplemented!()
    }

    pub fn events(&self) -> Events {
        Events { window: self }
    }

    pub fn clear(&self) {
        unimplemented!()
    }

    pub fn flip(&mut self) {
        unimplemented!()
    }
}

pub struct Events<'a> {
    window: &'a Window,
}

impl Iterator for Events<'_> {
    type Item = super::WindowEvent;
    fn next(&mut self) -> Option<super::WindowEvent> {
        unimplemented!()
    }
}
