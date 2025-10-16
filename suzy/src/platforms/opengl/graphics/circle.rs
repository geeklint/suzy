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
    pub inner_radius: f32,
}

impl Default for Circle {
    fn default() -> Self {
        Self {
            texture: Texture::default(),
            color: Color::WHITE,
            center: [50.0, 50.0],
            outer_radius: 50.0,
            inner_radius: 0.0,
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
            left: self.center[0] - self.outer_radius - 0.5,
            right: self.center[0] + self.outer_radius + 0.5,
            bottom: self.center[1] - self.outer_radius - 0.5,
            top: self.center[1] + self.outer_radius + 0.5,
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
                        match [
                            precise_midpoint(
                                uv_rect_u16.left,
                                uv_rect_u16.right,
                            ),
                            precise_midpoint(
                                uv_rect_u16.bottom,
                                uv_rect_u16.top,
                            ),
                        ] {
                            [Some(cu), Some(cv)] => {
                                self.push_vertices(
                                    batch,
                                    bbox,
                                    uv_rect_u16,
                                    [cu, cv],
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
        // add half a pixel so the middle of the antialiased edge is at the
        // outer radius
        let outer_radius_aa = self.outer_radius + 0.5;
        let peak = if self.inner_radius == 0.0 {
            1.0
        } else {
            // the peak needs to be half-way between the outer edge and the
            // inner edge, mesured backwards because the computed alpha
            // decreases with more distance.
            let outer_radius_norm =
                1.0 - (self.outer_radius / outer_radius_aa);
            let inner_radius_norm =
                1.0 - (self.inner_radius / outer_radius_aa);
            inner_radius_norm.midpoint(outer_radius_norm)
        };
        let config = VertexConfig::new().alpha_base(0.0).alpha_peak(peak);
        let smoothing = outer_radius_aa;
        for (y, v, y_inside) in vertical_values {
            for &(x, u, x_inside) in &horiz_values {
                let config = config.vector(x_inside, y_inside);
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

fn precise_midpoint(a: u16, b: u16) -> Option<u16> {
    if a.abs_diff(b).is_multiple_of(2) {
        Some(a.midpoint(b))
    } else {
        None
    }
}
