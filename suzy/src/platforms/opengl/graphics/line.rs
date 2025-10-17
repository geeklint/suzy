use std::convert::TryInto;

use crate::{
    graphics::{DrawContext, Graphic},
    platforms::opengl,
};

use opengl::{
    renderer::{
        Batch, BatchRef, BoundingBox, UvRect, UvRectValues, UvType, Vertex,
        VertexConfig,
    },
    OpenGlRenderPlatform, Texture,
};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Line<Color = crate::graphics::Color> {
    pub color: Color,
    pub points: Vec<[f32; 2]>,
    pub width: f32,
    pub close_loop: bool,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            color: crate::graphics::Color::WHITE,
            points: vec![[0.0, 0.0], [100.0, 100.0]],
            width: 2.0,
            close_loop: false,
        }
    }
}

impl<Color> Line<Color> {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            points: vec![[0.0, 0.0], [100.0, 100.0]],
            width: 2.0,
            close_loop: false,
        }
    }
}

impl<Color> Graphic<OpenGlRenderPlatform> for Line<Color>
where
    Color: LinePainter,
{
    fn draw(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {
        if self.points.is_empty() {
            // no points - no line to draw
            return;
        }
        let bbox = self.points.iter().fold(
            BoundingBox::default(),
            |bbox, &[x, y]| {
                bbox.merge(&BoundingBox {
                    left: x,
                    right: x,
                    bottom: y,
                    top: y,
                })
            },
        );
        let num_vertices = self
            .points
            .len()
            .try_into()
            .unwrap_or(u16::MAX)
            .saturating_mul(4);
        let Some(BatchRef { batch, mut uv_rect }) =
            ctx.find_batch(self.color.texture(), num_vertices, &[bbox])
        else {
            return;
        };
        loop {
            match uv_rect {
                UvRect::SolidColor(u, v) => {
                    let rect = UvRectValues {
                        left: u,
                        right: u,
                        bottom: v,
                        top: v,
                    };
                    if self.color.can_use_short_uvs(rect) {
                        self.push_vertices(batch, rect);
                        return;
                    } else {
                        uv_rect = UvRect::F32(UvRectValues {
                            left: u.to_f32(),
                            right: u.to_f32(),
                            bottom: v.to_f32(),
                            top: v.to_f32(),
                        });
                        continue;
                    }
                }
                UvRect::F32(uv_rect_f32) => {
                    self.push_vertices(batch, uv_rect_f32);
                    return;
                }
                UvRect::U16(uv_rect_u16) => {
                    if self.color.can_use_short_uvs(uv_rect_u16) {
                        self.push_vertices(batch, uv_rect_u16);
                        return;
                    } else {
                        uv_rect = UvRect::F32(UvRectValues {
                            left: uv_rect_u16.left.to_f32(),
                            right: uv_rect_u16.right.to_f32(),
                            bottom: uv_rect_u16.bottom.to_f32(),
                            top: uv_rect_u16.top.to_f32(),
                        });
                        continue;
                    }
                }
            }
        }
    }
}

impl<Color> Line<Color>
where
    Color: LinePainter,
{
    fn push_vertices<Uv>(&self, batch: &mut Batch, uv_rect: UvRectValues<Uv>)
    where
        Uv: UvType,
    {
        let index_offset: u16 = batch.vertices.len_u16();
        let mut points_iter = self.points.iter().copied();
        let [first_point, last_point] =
            match [points_iter.next(), points_iter.next_back()] {
                [Some(a), Some(b)] => [a, b],
                [Some(a), None] | [None, Some(a)] => [a, a],
                [None, None] => {
                    // no points - no line to draw
                    return;
                }
            };
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct LineVertex {
    pub xy: [f32; 2],
    pub distance: f32,
    pub progress: f32,
}

pub trait LinePainter {
    fn texture(&self) -> &Texture;
    fn can_use_short_uvs(&self, rect: UvRectValues<u16>) -> bool;
    fn paint_vertex<Uv>(
        &self,
        rect: UvRectValues<Uv>,
        vertex: LineVertex,
    ) -> Vertex<Uv>;
}

impl LinePainter for crate::graphics::Color {
    fn texture(&self) -> &Texture {
        const { &Texture::solid_color() }
    }

    fn can_use_short_uvs(&self, _rect: UvRectValues<u16>) -> bool {
        true
    }

    fn paint_vertex<Uv>(
        &self,
        rect: UvRectValues<Uv>,
        vertex: LineVertex,
    ) -> Vertex<Uv> {
        Vertex {
            xy: vertex.xy,
            uv: [rect.left, rect.bottom],
            color: self.rgba8(),
            config: VertexConfig::new(),
            smoothing: 1.0,
        }
    }
}
