/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::convert::TryInto;

use crate::{
    graphics::{Color, DrawContext, Graphic},
    text,
    watch::WatchedMeta,
};

use super::{
    context::bindings::{FALSE, FLOAT, TRIANGLES},
    renderer::{BatchPool, BatchRef, BoundingBox, Vertex},
    texture::Texture,
    OpenGlRenderPlatform,
};

mod calc;
mod font;

use calc::{CalcParams, Cursor, FontCharCalc};
pub use font::Font;

#[cfg(feature = "default_font")]
fn default_font() -> &'static font::Font {
    todo!()
}

#[cfg(not(feature = "default_font"))]
#[track_caller]
fn default_font() -> &'static font::Font {
    panic!("invalid font index specified, and no default font is available");
}

pub struct TextStyle {
    pub font_size: f32,
    pub color: Color,
    pub font: usize,
}

impl crate::platform::graphics::TextStyle for TextStyle {
    fn with_size_and_color(size: f32, color: Color) -> Self {
        Self {
            font_size: size,
            color,
            font: 0,
        }
    }

    fn push_tag(&self, tag: &mut &str) -> Result<Self, ()> {
        Err(())
    }
}

/// Default Graphic for displaying Text.
///
/// This implementation is based on a signed distance field font atlas, these
/// fonts can be generated using the crate `suzy_build_tools`.
#[derive(Default)]
pub struct Text {
    fonts: Vec<font::Font>,
    vertices: Vec<VertexSet>,
    layout_changed: WatchedMeta<'static>,
    calc: FontCharCalc,
}

impl Graphic<OpenGlRenderPlatform> for Text {
    fn draw(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {
        for vs in &mut self.vertices {
            let vs_vertices = &vs.vertices;
            let &mut bbox = vs.bounding_box.get_or_insert_with(|| {
                let mut bbox = BoundingBox {
                    left: f32::INFINITY,
                    right: f32::NEG_INFINITY,
                    bottom: f32::INFINITY,
                    top: f32::NEG_INFINITY,
                };
                for vertex in vs_vertices {
                    bbox.left = bbox.left.min(vertex.xy[0]);
                    bbox.right = bbox.right.max(vertex.xy[0]);
                    bbox.bottom = bbox.bottom.min(vertex.xy[1]);
                    bbox.top = bbox.top.max(vertex.xy[1]);
                }
                bbox
            });
            if let Some(BatchRef { batch, uv_rect }) = ctx.find_batch(
                &vs.texture,
                vs.vertices.len().try_into().expect(
            "the number of vertices in a text object should be less than 2^16",
                ),
                &[bbox],
            ) {
                let index_offset: u16 =
                    batch.vertices.len().try_into().expect(
            "the number of vertices in a batch should be less than 2^16",
                    );
                for &vertex in &vs.vertices {
                    batch.vertices.push(vertex);
                }
                batch
                    .indices
                    .extend(vs.indices.iter().map(|idx| idx + index_offset));
            }
        }
    }
}

impl crate::platform::graphics::Text<TextStyle> for Text {
    fn set_layout(&mut self, layout: text::Layout) {
        self.calc.layout = layout;
        self.layout_changed.trigger_auto();
    }

    fn clear(&mut self) {
        self.layout_changed.watched_auto();
        self.calc.cursor = Cursor::default();
        self.vertices.clear();
    }

    fn push_span(&mut self, style: TextStyle, text: &str) {
        self.calc.cursor.font_size = style.font_size;
        let font = self
            .fonts
            .get(style.font)
            .unwrap_or_else(|| default_font())
            .as_ref();
        let color = style.color.rgba8();
        let texture = font.data().texture.clone();
        let vertex_set_index = match self
            .vertices
            .iter_mut()
            .enumerate()
            .find(|(_, vs)| vs.texture.id() == texture.id())
        {
            Some((idx, _)) => idx,
            None => {
                let idx = self.vertices.len();
                let vs = VertexSet {
                    texture,
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    line_start_index: 0,
                    bounding_box: None,
                };
                self.vertices.push(vs);
                idx
            }
        };
        let mut remaining = text;
        while !remaining.is_empty() {
            let vertices = &mut self.vertices;
            let mut params = CalcParams {
                font,
                handle_glyph: &mut |glyph: calc::GlyphMetrics| {
                    vertices[vertex_set_index].vertices.push(Vertex {
                        xy: [glyph.bb_left, glyph.bb_bottom],
                        uv: [glyph.tex_left, glyph.tex_bottom],
                        color,
                        config: todo!(),
                        smoothing: todo!(),
                    });
                    todo!()
                },
            };
            let (consumed, line_break) =
                self.calc.push_span(params.rbr(), text);
            remaining = &remaining[consumed..];
            if line_break {
                for vs in &mut self.vertices {
                    self.calc
                        .align_line(vs.vertices.iter_mut().map(|v| &mut v.xy));
                }
                self.calc.reset_line();
            }
        }
    }

    fn finish(&mut self) {
        for vs in &mut self.vertices {
            self.calc
                .align_block(vs.vertices.iter_mut().map(|v| &mut v.xy));
        }
    }
}

struct VertexSet {
    texture: Texture,
    vertices: Vec<Vertex<u16>>,
    indices: Vec<u16>,
    line_start_index: usize,
    bounding_box: Option<BoundingBox>,
}
