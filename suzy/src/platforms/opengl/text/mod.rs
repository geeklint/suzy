/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{convert::TryInto, rc::Rc};

use crate::{
    graphics::{Color, DrawContext, Graphic},
    text,
    watch::WatchedMeta,
};

use super::{
    renderer::{BatchRef, BoundingBox, Vertex, VertexConfig},
    texture::Texture,
    OpenGlRenderPlatform,
};

mod calc;
mod font;

use calc::{CalcParams, FontCharCalc};
pub use font::Font;

#[cfg(feature = "default_font")]
mod default_font;

#[cfg(not(feature = "default_font"))]
mod default_font {
    #[track_caller]
    pub fn default_font() -> Rc<font::Font> {
        panic!(
            "invalid font index specified, and no default font is available"
        );
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Layer {
    Shadow,
    #[default]
    Primary,
}

#[derive(Clone, Debug)]
struct Draw {
    layer: Layer,
    color: Color,
    midpoint: f32,
    peak: f32,
    smoothing: f32,
}

#[derive(Clone, Debug)]
pub struct TextStyle {
    pub font_size: f32,
    pub font: usize,
    draws: Vec<Draw>,
}

impl crate::platform::graphics::TextStyle for TextStyle {
    fn with_size_and_color(size: f32, color: Color) -> Self {
        Self {
            font_size: size,
            font: 0,
            draws: vec![Draw {
                layer: Layer::Primary,
                color,
                midpoint: 0.5,
                peak: 1.0,
                smoothing: f32::NAN,
            }],
        }
    }

    fn push_tag(
        &self,
        tag: &mut &str,
    ) -> Result<Self, text::RichTextTagParseError> {
        Err(text::RichTextTagParseError {
            msg: format!("unknown tag: {tag}"),
        })
    }
}

/// Default Graphic for displaying Text.
///
/// This implementation is based on a signed distance field font atlas, these
/// fonts can be generated using the crate `suzy_build_tools`.
#[derive(Default)]
pub struct Text {
    fonts: Vec<Rc<font::Font>>,
    vertices: Vec<VertexSet>,
    layout_changed: WatchedMeta<'static>,
    calc: FontCharCalc,
}

impl Text {
    fn finish_line(&mut self) {
        for vs in &mut self.vertices {
            self.calc.align_line(
                vs.vertices
                    .iter_mut()
                    .skip(vs.line_start_index)
                    .map(|v| &mut v.xy),
            );
            vs.line_start_index = vs.vertices.len();
        }
        self.calc.reset_line();
    }
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
            if let Some(BatchRef { batch, .. }) =
                ctx.find_batch(&vs.texture, vs.vertices.len_u16(), &[bbox])
            {
                let index_offset: u16 = batch.vertices.len_u16();
                for &vertex in &vs.vertices {
                    batch.vertices.push(vertex);
                }
                batch.indices.extend(
                    vs.indices
                        .make_final()
                        .iter()
                        .map(|idx| idx + index_offset),
                );
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
        self.calc.reset();
        self.vertices.clear();
    }

    fn push_span(&mut self, style: TextStyle, text: &str) {
        self.calc.cursor.font_size = style.font_size;
        let font_clone = self
            .fonts
            .get(style.font)
            .cloned()
            .unwrap_or_else(default_font::default_font);
        let font = font_clone.as_ref();
        let aa_smoothing = style.font_size * (2.0 * font.data.padding_ratio);
        let draws = &style.draws;
        let texture = font.data.texture.clone();
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
                    indices: IndicesState::Unsorted(Vec::new()),
                    line_start_index: 0,
                    bounding_box: None,
                };
                self.vertices.push(vs);
                idx
            }
        };
        let mut remaining = text;
        while !remaining.is_empty() {
            let vertex_set = &mut self.vertices[vertex_set_index];
            let vertices = &mut vertex_set.vertices;
            let indices = vertex_set.indices.unsorted();
            let params = CalcParams {
                font: &font.data,
                handle_glyph: &mut |glyph: calc::GlyphMetrics| {
                    for draw in draws {
                        let color = draw.color.rgba8();
                        let smoothing = if draw.smoothing.is_nan() {
                            aa_smoothing
                        } else {
                            draw.smoothing
                        };
                        let base = draw.midpoint - 0.5 / smoothing;
                        let magic = 255.0 / 2.0;
                        let offset = (-magic * base + magic).clamp(0.0, 255.0);
                        let peak = (draw.peak * 255.0).clamp(0.0, 255.0);
                        let config =
                            VertexConfig([offset as u8, peak as u8, 0, 0]);
                        let left = (glyph.bb_left, glyph.tex_left);
                        let right = (glyph.bb_right, glyph.tex_right);
                        let bottom = (glyph.bb_bottom, glyph.tex_bottom);
                        let top = (glyph.bb_top, glyph.tex_top);
                        indices.push(draw.layer);
                        for (y, v) in [bottom, top] {
                            for (x, u) in [left, right] {
                                vertices.push(Vertex {
                                    xy: [x, y],
                                    uv: [u, v],
                                    color,
                                    config,
                                    smoothing,
                                });
                            }
                        }
                    }
                },
            };
            let (consumed, line_break) =
                self.calc.push_span(params, remaining);
            remaining = &remaining[consumed..];
            if line_break {
                self.finish_line();
            }
        }
    }

    fn finish(&mut self) {
        self.finish_line();
        for vs in &mut self.vertices {
            self.calc
                .align_block(vs.vertices.iter_mut().map(|v| &mut v.xy));
            vs.indices.make_final();
        }
    }
}

#[derive(Clone, Debug)]
enum IndicesState {
    Unsorted(Vec<Layer>),
    Final(Vec<u16>),
}

impl IndicesState {
    fn unsorted(&mut self) -> &mut Vec<Layer> {
        match self {
            IndicesState::Unsorted(vec) => vec,
            IndicesState::Final(_) => {
                unreachable!("push_span called after finish but before clear")
            }
        }
    }

    fn make_final(&mut self) -> &mut Vec<u16> {
        loop {
            match self {
                IndicesState::Final(vec) => return vec,
                IndicesState::Unsorted(layer_vec) => {
                    let vertex_count = layer_vec.len_u16();
                    let mut layer_index_vec: Vec<u16> =
                        (0..vertex_count).collect();
                    layer_index_vec
                        .sort_by_key(|index| &layer_vec[usize::from(*index)]);
                    *self = IndicesState::Final(
                        layer_index_vec
                            .into_iter()
                            .flat_map(|layer_index| {
                                let bl = layer_index * 4;
                                let br = bl + 1;
                                let tl = bl + 2;
                                let tr = bl + 3;
                                [bl, br, tl, br, tr, tl]
                            })
                            .collect(),
                    );
                }
            }
        }
    }
}

struct VertexSet {
    texture: Texture,
    vertices: Vec<Vertex<u16>>,
    indices: IndicesState,
    line_start_index: usize,
    bounding_box: Option<BoundingBox>,
}

trait ExpectU16Len {
    fn len_u16(&self) -> u16;
}

impl<T> ExpectU16Len for Vec<T> {
    fn len_u16(&self) -> u16 {
        self.len().try_into().expect(
            "the number of vertices in a text object should be less than 2^16",
        )
    }
}
