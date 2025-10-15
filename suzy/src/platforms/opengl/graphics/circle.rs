use std::convert::TryInto;

use crate::{
    graphics::{Color, DrawContext, Graphic},
    platforms::opengl::{self, renderer::Vertex},
};

use opengl::{
    renderer::{
        Batch, BatchRef, BoundingBox, UvRect, UvRectValues, UvType,
        VertexConfig,
    },
    OpenGlRenderPlatform, Texture,
};

#[derive(Clone, Debug)]
pub struct Circle {
    pub texture: Texture,
    pub color: Color,
    pub center: [f32; 2],
    pub outer_radius: f32,
}

impl Default for Circle {
    fn default() -> Self {
        Self {
            texture: Texture::default(),
            color: Color::WHITE,
            center: [50.0, 50.0],
            outer_radius: 50.0,
        }
    }
}

impl Circle {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Graphic<OpenGlRenderPlatform> for Circle {
    fn draw(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {
        let bbox = BoundingBox {
            left: self.center[0] - self.outer_radius,
            right: self.center[0] + self.outer_radius,
            bottom: self.center[1] - self.outer_radius,
            top: self.center[1] + self.outer_radius,
        };
        if let Some(BatchRef { batch, mut uv_rect }) =
            ctx.find_batch(&self.texture, 9, &[bbox])
        {
            loop {
                match uv_rect {
                    UvRect::SolidColor(u, v) => {
                        let rect = UvRectValues {
                            left: u,
                            right: u,
                            bottom: v,
                            top: v,
                        };
                        self.push_vertices(batch, bbox, rect, [u, v]);
                        return;
                    }
                    UvRect::F32(uv_rect_f32) => {
                        let uv_center = [
                            uv_rect_f32.left.midpoint(uv_rect_f32.right),
                            uv_rect_f32.bottom.midpoint(uv_rect_f32.top),
                        ];
                        self.push_vertices(
                            batch,
                            bbox,
                            uv_rect_f32,
                            uv_center,
                        );
                        return;
                    }
                    UvRect::U16(uv_rect_u16) => {
                        if (uv_rect_u16.right - uv_rect_u16.left) & 1 == 0
                            || (uv_rect_u16.top - uv_rect_u16.bottom) & 1 == 0
                        {
                            // dims are even, so they can't be midpoint-ed
                            uv_rect = UvRect::F32(UvRectValues {
                                left: uv_rect_u16.left.to_f32(),
                                right: uv_rect_u16.right.to_f32(),
                                bottom: uv_rect_u16.bottom.to_f32(),
                                top: uv_rect_u16.top.to_f32(),
                            });
                            continue;
                        }
                        let uv_center = [
                            uv_rect_u16.left.midpoint(uv_rect_u16.right),
                            uv_rect_u16.bottom.midpoint(uv_rect_u16.top),
                        ];
                        self.push_vertices(
                            batch,
                            bbox,
                            uv_rect_u16,
                            uv_center,
                        );
                        return;
                    }
                }
            }
        }
    }
}

impl Circle {
    fn push_vertices<Uv>(
        &self,
        batch: &mut Batch,
        bbox: BoundingBox,
        uv_rect: UvRectValues<Uv>,
        uv_center: [Uv; 2],
    ) where
        Uv: UvType,
    {
        let index_offset: u16 = batch.vertices.len().try_into().expect(
            "the number of vertices in a batch should be less than 2^16",
        );
        let horiz_values = [
            (bbox.left, uv_rect.left, false),
            (self.center[0], uv_center[0], true),
            (bbox.right, uv_rect.right, false),
        ];
        let vertical_values = [
            (bbox.bottom, uv_rect.bottom, false),
            (self.center[1], uv_center[1], true),
            (bbox.top, uv_rect.top, false),
        ];
        let color = self.color.rgba8();
        for (y, v, y_inside) in vertical_values {
            for &(x, u, x_inside) in &horiz_values {
                let smoothing = self.outer_radius;
                let config = VertexConfig::new()
                    .alpha_base(0.0)
                    .vector(x_inside, y_inside);
                batch.vertices.push(Vertex {
                    xy: [x, y],
                    uv: [u, v],
                    color,
                    config,
                    smoothing,
                });
            }
        }
        batch
            .indices
            .extend(INDICES.iter().map(|&i| u16::from(i) + index_offset));
    }
}

// 6  7  8
// 3  4  5
// 0  1  2

#[rustfmt::skip]
static INDICES: [u8; 8 * 3] = [
    0, 1, 3,
    1, 4, 3,
    1, 2, 4,
    2, 5, 4,
    3, 4, 6,
    4, 7, 6,
    4, 5, 7,
    5, 8, 7,
];
