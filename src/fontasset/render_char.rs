use rusttype::Font;

use crate::sdf;
use super::output::{FontOutput, GlyphMetric};

pub(super) struct CharToRender {
    pub ch: char,
    pub dest_scale: rusttype::Scale,
    pub ref_scale: rusttype::Scale,
    pub x: f64,
    pub y: f64,
    pub chan: usize,
    pub padding: f64,
    pub norm_padding: f32,
}

pub(super) struct DestBuffer<'a> {
    pub buffer: &'a mut FontOutput,
    pub tex_size: usize,
    pub nchans: usize,
}


pub(super) fn render_char(
    font: &Font,
    ch: CharToRender,
    buf: &mut DestBuffer,
) {
    let one = rusttype::Scale::uniform(1.0);
    let zero = rusttype::Point { x: 0.0, y: 0.0 };
    let bb = font.glyph(ch.ch)
        .scaled(ch.dest_scale)
        .exact_bounding_box()
        .expect("no bounding box for ?");
    let norm_glyph = font.glyph(ch.ch).scaled(one);
    let norm_bb = norm_glyph.exact_bounding_box()
        .expect("no bounding box for ?");
    let sprite_width = (bb.width() as f64 + 2.0 * ch.padding).floor();
    let sprite_height = (bb.height() as f64 + 2.0 * ch.padding).floor();
    buf.buffer.add_metric(GlyphMetric {
        ch: ch.ch,
        u_offset: ch.x as f32,
        v_offset: ch.y as f32,
        uv_width: sprite_width as f32 / buf.tex_size as f32,
        uv_height: sprite_height as f32 / buf.tex_size as f32,
        bb_min_x: norm_bb.min.x - ch.norm_padding,
        bb_max_x: norm_bb.max.x + ch.norm_padding,
        // y-coordinates are opposite what I think they are?
        bb_min_y: -norm_bb.max.y - ch.norm_padding,
        bb_max_y: -norm_bb.min.y + ch.norm_padding,
        advance_width: norm_glyph.h_metrics().advance_width,
    });

    let glyph = font.glyph(ch.ch).scaled(ch.ref_scale).positioned(zero);
    let pxbb = glyph.pixel_bounding_box().expect("no pixel bounding box?");
    let bm_bufsize = (pxbb.width() * pxbb.height()) as usize;
    let pitch = pxbb.width() as usize;
    let mut bitmap = vec![false; bm_bufsize];
    glyph.draw(|x, y, v| {
        let x = x as usize;
        let y = y as usize;
        bitmap[y * pitch + x] = v >= 0.5;
    });
    let x_offset = (ch.x * (buf.tex_size as f64)).floor() as usize;
    let y_offset = (ch.y * (buf.tex_size as f64)).floor() as usize;
    let source = sdf::SourceBitmap {
        buffer: &bitmap,
        width: pxbb.width() as usize,
        height: pxbb.height() as usize,
    };
    let dest = sdf::DestImage {
        buf: sdf::DestBuffer {
            buffer: buf.buffer.buffer(),
            width: buf.tex_size,
            height: buf.tex_size,
            num_channels: buf.nchans,
        },
        padding: ch.padding,
        channel: ch.chan,
        x_offset,
        y_offset,
        width: sprite_width as usize,
        height: sprite_height as usize,
    };
    sdf::render_sdf(dest, source);
}

