/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::convert::TryInto;

use crate::{
    dims::{Dim, Padding2d, Rect, SimplePadding2d, SimpleRect},
    graphics::{Color, CornerStyle, DrawContext, Graphic},
    platforms::opengl,
};

use opengl::{
    renderer::{
        Batch, BatchRef, UvRect, UvRectValues, UvType, Vertex, VertexConfig,
    },
    OpenGlRenderPlatform, Texture,
};

// 12 13 14 15
//  8  9 10 11
//  4  5  6  7
//  0  1  2  3

#[rustfmt::skip]
static SLICED_INDICES: [u8; 18 * 3] = [
    0, 1, 4,
    1, 5, 4,
    1, 2, 5,
    2, 6, 5,
    2, 3, 6,
    3, 7, 6,
    4, 5, 8,
    5, 9, 8,
    5, 6, 9,
    6, 10, 9,
    6, 7, 10,
    7, 11, 10,
    8, 9, 12,
    9, 13, 12,
    9, 10, 13,
    10, 14, 13,
    10, 11, 14,
    11, 15, 14,
];

/// A common graphic used for user interfaces, a sliced image is defined by
/// fixed-sized corners and an inner area which stretches to fill the
/// graphic area.
///
/// See the [Wikipedia article](https://en.wikipedia.org/wiki/9-slice_scaling)
/// on 9-slice scaling for more information.
pub struct SlicedImage {
    pub padding: SimplePadding2d,
    pub texture: Texture,
    color: Color,
    corners: CornerStyle,
    rect: SimpleRect,
}

impl Default for SlicedImage {
    fn default() -> Self {
        Self {
            padding: SimplePadding2d::default(),
            texture: Texture::default(),
            color: Color::WHITE,
            corners: CornerStyle::NotRounded,
            rect: SimpleRect::default(),
        }
    }
}

impl SlicedImage {
    /// Create a new SlicedImage.
    pub fn new() -> Self {
        Self::default()
    }
}

impl crate::platform::graphics::SlicedImage for SlicedImage {
    fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    fn set_slice_padding(&mut self, padding: impl Padding2d) {
        self.padding = (&padding).into();
    }

    fn set_corners(&mut self, style: CornerStyle) {
        self.corners = style;
    }
}

impl Rect for SlicedImage {
    fn x(&self) -> Dim {
        self.rect.x()
    }
    fn y(&self) -> Dim {
        self.rect.y()
    }

    fn x_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R,
    {
        self.rect.x_mut(f)
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R,
    {
        self.rect.y_mut(f)
    }
}

impl Graphic<OpenGlRenderPlatform> for SlicedImage {
    fn draw(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {
        if let Some(BatchRef { batch, mut uv_rect }) =
            ctx.find_batch(&self.texture, 16, &[(&self.rect).into()])
        {
            loop {
                match uv_rect {
                    UvRect::F32(uv_rect_f32) => {
                        let inner_uv_rect = UvRectValues {
                            left: uv_rect_f32.left + self.padding.left(),
                            right: uv_rect_f32.right - self.padding.right(),
                            bottom: uv_rect_f32.bottom + self.padding.bottom(),
                            top: uv_rect_f32.top - self.padding.top(),
                        };
                        self.push_vertices(batch, uv_rect_f32, inner_uv_rect);
                        return;
                    }
                    UvRect::U16(uv_rect_u16) => {
                        match [
                            self.padding.left(),
                            self.padding.right(),
                            self.padding.bottom(),
                            self.padding.top(),
                        ]
                        .map(u16::try_from_f32)
                        {
                            [Some(left), Some(right), Some(bottom), Some(top)] =>
                            {
                                let inner_uv_rect = UvRectValues {
                                    left: uv_rect_u16.left + left,
                                    right: uv_rect_u16.right - right,
                                    bottom: uv_rect_u16.bottom + bottom,
                                    top: uv_rect_u16.top - top,
                                };
                                self.push_vertices(
                                    batch,
                                    uv_rect_u16,
                                    inner_uv_rect,
                                );
                                return;
                            }
                            _ => {
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
    }
}

impl SlicedImage {
    fn push_vertices<Uv>(
        &self,
        batch: &mut Batch,
        uv_rect: UvRectValues<Uv>,
        inner_uv_rect: UvRectValues<Uv>,
    ) where
        Uv: UvType,
    {
        let rect = &self.rect;
        let index_offset: u16 = batch.vertices.len().try_into().expect(
            "the number of vertices in a batch should be less than 2^16",
        );
        let mut inner = SimpleRect::default();
        inner.set_fill(rect, &self.padding);
        let (outside, pad_left, pad_right, pad_bottom, pad_top);
        if matches!(self.corners, CornerStyle::Rounded) {
            outside = false;
            pad_left = self.padding.left();
            pad_right = self.padding.right();
            pad_bottom = self.padding.bottom();
            pad_top = self.padding.top();
        } else {
            outside = true;
            pad_left = 1.0;
            pad_right = 1.0;
            pad_bottom = 1.0;
            pad_top = 1.0;
        };
        let horiz_values = [
            (rect.left(), uv_rect.left, outside, pad_left),
            (inner.left(), inner_uv_rect.left, true, pad_left),
            (inner.right(), inner_uv_rect.right, true, pad_right),
            (rect.right(), uv_rect.right, outside, pad_right),
        ];
        let vertical_values = [
            (rect.bottom(), uv_rect.bottom, outside, pad_bottom),
            (inner.bottom(), inner_uv_rect.bottom, true, pad_bottom),
            (inner.top(), inner_uv_rect.top, true, pad_top),
            (rect.top(), uv_rect.top, outside, pad_top),
        ];
        let color = self.color.rgba8();
        let mut odd = false;
        for (y, v, y_inside, y_padding) in vertical_values {
            for &(x, u, x_inside, x_padding) in &horiz_values {
                let smoothing = match (x_inside, y_inside) {
                    (true, false) => x_padding,
                    (false, true) => y_padding,
                    _ => (x_padding + y_padding) / 2.0,
                };
                let config = VertexConfig::new()
                    .alpha_base(0.000001)
                    .vector(x_inside, y_inside);
                batch.vertices.push(Vertex {
                    xy: [x, y],
                    uv: [u, v],
                    color,
                    config,
                    smoothing,
                });
                odd = !odd;
            }
            odd = !odd;
        }
        batch.indices.extend(
            SLICED_INDICES.iter().map(|&i| u16::from(i) + index_offset),
        );
    }
}
