/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2025 Violet Leonard */

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
        let mut points_iter = self.points.iter();
        let [first_point, last_point] =
            match [points_iter.next(), points_iter.as_slice().last()] {
                [Some(&a), Some(&b)] => [a, b],
                [Some(&a), None] | [None, Some(&a)] => [a, a],
                [None, None] => {
                    // no points - no line to draw
                    return;
                }
            };
        let tail;
        let mut prev_point = if self.close_loop {
            tail = first_point;
            last_point
        } else {
            tail = last_point;
            let second_point =
                *points_iter.as_slice().first().unwrap_or(&last_point);
            let dp2 = [
                second_point[0] - first_point[0],
                second_point[1] - first_point[1],
            ];
            [first_point[0] - dp2[0], first_point[1] - dp2[1]]
        };
        let half_width = self.width * 0.5;
        let (mut prev_left, mut prev_right);
        {
            let dx = first_point[0] - prev_point[0];
            let dy = first_point[1] - prev_point[1];
            let length = (dx.powi(2) + dy.powi(2)).sqrt();
            let half_aspect = half_width / length;
            let left_offset = [-half_aspect * dy, half_aspect * dx];
            let right_offset = [half_aspect * dy, -half_aspect * dx];
            prev_left = [
                vadd(prev_point, left_offset),
                vadd(first_point, left_offset),
            ];
            prev_right = [
                vadd(prev_point, right_offset),
                vadd(first_point, right_offset),
            ];
        }
        prev_point = first_point;
        let mut prev_indicies: Option<[u16; 2]> = None;
        let mut first_indicies: Option<[u16; 2]> = None;
        let mut distance_accum = 0.0;
        for point in points_iter.copied().chain([tail]) {
            if point == prev_point {
                continue;
            }
            let dx = point[0] - prev_point[0];
            let dy = point[1] - prev_point[1];
            let length = (dx.powi(2) + dy.powi(2)).sqrt();
            let half_aspect = half_width / length;
            let left_offset = [-half_aspect * dy, half_aspect * dx];
            let right_offset = [half_aspect * dy, -half_aspect * dx];
            let left =
                [vadd(prev_point, left_offset), vadd(point, left_offset)];
            let right =
                [vadd(prev_point, right_offset), vadd(point, right_offset)];
            let left_intersect = intersection(prev_left, left, left[0]);
            let right_intersect = intersection(prev_right, right, right[0]);
            let idx_left = batch.vertices.push(self.color.paint_vertex(
                uv_rect,
                LineVertex {
                    xy: left_intersect,
                    distance: distance_accum,
                    chiral: -1.0,
                },
            ));
            let idx_right = batch.vertices.push(self.color.paint_vertex(
                uv_rect,
                LineVertex {
                    xy: right_intersect,
                    distance: distance_accum,
                    chiral: 1.0,
                },
            ));
            if let Some([pil, pir]) = prev_indicies {
                batch.indices.extend([
                    pil, pir, idx_left, //
                    pir, idx_right, idx_left,
                ]);
            } else {
                first_indicies = Some([idx_left, idx_right]);
            }
            prev_point = point;
            prev_left = left;
            prev_right = right;
            prev_indicies = Some([idx_left, idx_right]);
            distance_accum += length;
        }
        if let [Some([pil, pir]), Some([fil, fir])] =
            [prev_indicies, first_indicies]
        {
            if self.close_loop {
                batch.indices.extend([
                    pil, pir, fil, //
                    pir, fir, fil,
                ]);
            } else {
                let idx_left = batch.vertices.push(self.color.paint_vertex(
                    uv_rect,
                    LineVertex {
                        xy: prev_left[1],
                        distance: distance_accum,
                        chiral: -1.0,
                    },
                ));
                let idx_right = batch.vertices.push(self.color.paint_vertex(
                    uv_rect,
                    LineVertex {
                        xy: prev_right[1],
                        distance: distance_accum,
                        chiral: 1.0,
                    },
                ));
                batch.indices.extend([
                    pil, pir, idx_left, //
                    pir, idx_right, idx_left,
                ]);
            }
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct LineVertex {
    pub xy: [f32; 2],
    pub distance: f32,
    pub chiral: f32,
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

fn vadd(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

fn intersection(a: [[f32; 2]; 2], b: [[f32; 2]; 2], or: [f32; 2]) -> [f32; 2] {
    let dxa = a[0][0] - a[1][0];
    let dya = a[0][1] - a[1][1];
    let dxb = b[0][0] - b[1][0];
    let dyb = b[0][1] - b[1][1];
    let denom = dxa * dyb - dya * dxb;
    // this epsilon was chosen somewhat arbitrarily
    if denom.abs() < 0.01 {
        return or;
    }
    let aq = a[0][0] * a[1][1] - a[0][1] * a[1][0];
    let bq = b[0][0] * b[1][1] - b[0][1] * b[1][0];
    let num_x = aq * dxb - bq * dxa;
    let num_y = aq * dyb - bq * dya;
    [num_x / denom, num_y / denom]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection() {
        let x_intersect = intersection(
            [[0.0, 0.0], [1.0, 1.0]],
            [[0.0, 1.0], [1.0, 0.0]],
            [-1.0, -1.0],
        );
        assert_eq!(x_intersect, [0.5, 0.5]);
    }
}
