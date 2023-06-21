pub struct Glyph {
    pub ch: char,
    pub advance: f32,
    pub bb_left: f32,
    pub bb_right: f32,
    pub bb_bottom: f32,
    pub bb_top: f32,
    pub tex_left: u16,
    pub tex_right: u16,
    pub tex_bottom: u16,
    pub tex_top: u16,
}

include! {"default_font.rs"}

pub static TEXTURE_DATA: &[u8] = include_bytes!("default_font.rs.texture");
