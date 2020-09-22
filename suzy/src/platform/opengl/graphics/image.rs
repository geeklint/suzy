/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::graphics::{DrawContext, Graphic};
use crate::dims::{Dim, Rect, SimpleRect, SimplePadding2d, Padding2d};

use crate::platform::opengl;
use opengl::context::bindings::{
    FALSE,
    FLOAT,
    TRIANGLES,
    UNSIGNED_BYTE,
};
use opengl::{
    OpenGlRenderPlatform,
    DualVertexBufferIndexed,
    Texture,
};

pub struct SlicedImage {
    rect: SimpleRect,
    padding: SimplePadding2d,
    texture: Texture,
    buffers: DualVertexBufferIndexed<f32>,
}

static SLICED_INDICES: [u8; 18 * 3] = [
    0, 4, 11,
    4, 12, 11,
    4, 5, 12,
    5, 13, 12,
    5, 1, 13,
    1, 6, 13,
    11, 12, 10,
    12, 15, 10,
    12, 13, 15,
    13, 14, 15,
    13, 6, 14,
    6, 7, 14,
    10, 15, 3,
    15, 9, 3,
    15, 14, 9,
    14, 8, 9,
    14, 7, 8,
    7, 2, 8,
];

impl Default for SlicedImage {
    fn default() -> Self {
        Self {
            rect: SimpleRect::default(),
            padding: SimplePadding2d::default(),
            texture: Texture::default(),
            buffers: DualVertexBufferIndexed::new(true, false, false),
        }
    }
}

impl SlicedImage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_image<P>(&mut self, texture: Texture, padding: &P)
    where
        P: Padding2d
    {
        self.texture = texture;
        self.padding = padding.into();
        self.update_image();
    }

    fn update_image(&mut self) {
        let mut uvs = [0f32; 32];
        let Self { buffers, texture, padding, .. } = self;
        buffers.set_data_1(|_gl| {
            texture.size().map(|(tex_width, tex_height)| {
                let left = padding.left() / tex_width;
                let right = 1.0 - (padding.right() / tex_width);
                let bottom = padding.bottom() / tex_height;
                let top = 1.0 - (padding.top() / tex_height);
                uvs = [
                    0.0, 0.0,
                    1.0, 0.0,
                    1.0, 1.0,
                    0.0, 1.0,
                    left, 0.0,
                    right, 0.0,
                    1.0, bottom,
                    1.0, top,
                    right, 1.0,
                    left, 1.0,
                    0.0, top,
                    0.0, bottom,
                    left, bottom,
                    right, bottom,
                    right, top,
                    left, top,
                ];
                &uvs[..]
            })
        });
    }

    fn update(&mut self) {
        let mut inner = SimpleRect::default();
        inner.set_fill(&self.rect, &self.padding);
        let rect = &self.rect;
        let mut vertices = [0f32; 32];
        self.buffers.set_data_0(|_gl| {
            vertices = [
                // outer corners
                rect.left(), rect.bottom(),
                rect.right(), rect.bottom(),
                rect.right(), rect.top(),
                rect.left(), rect.top(),
                // bottom edge
                inner.left(), rect.bottom(),
                inner.right(), rect.bottom(),
                // right edge
                rect.right(), inner.bottom(),
                rect.right(), inner.top(),
                // top edge
                inner.right(), rect.top(),
                inner.left(), rect.top(),
                // left edge
                rect.left(), inner.top(),
                rect.left(), inner.bottom(),
                // inner corners
                inner.left(), inner.bottom(),
                inner.right(), inner.bottom(),
                inner.right(), inner.top(),
                inner.left(), inner.top(),
            ];
            &vertices[..]
        });
    }
}

impl Rect for SlicedImage {
    fn x(&self) -> Dim { self.rect.x() }
    fn y(&self) -> Dim { self.rect.y() }

    fn x_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        let res = self.rect.x_mut(f);
        self.update();
        res
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        let res = self.rect.y_mut(f);
        self.update();
        res
    }
}

impl Graphic<OpenGlRenderPlatform> for SlicedImage {
    fn draw(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>) {
        ctx.push(|ctx| {
            ctx.params().standard_mode();
            ctx.params().use_texture(self.texture.clone());
            if let Some(ready) = self.buffers.check_ready(ctx) {
                let gl = ready.gl;
                ready.bind_0();
                unsafe {
                    gl.VertexAttribPointer(
                        0, 2, FLOAT, FALSE, 0, std::ptr::null(),
                    );
                }
                ready.bind_1();
                unsafe {
                    gl.VertexAttribPointer(
                        1, 2, FLOAT, FALSE, 0, std::ptr::null(),
                    );
                }
                ready.bind_indices();
                unsafe {
                    gl.DrawElements(
                        TRIANGLES,
                        SLICED_INDICES.len() as _,
                        UNSIGNED_BYTE,
                        std::ptr::null(),
                    );
                }
            } else {
                self.update();
                self.update_image();
                self.buffers.set_indices(|_gl| &SLICED_INDICES[..]);
                self.texture.bind(ctx.render_ctx_mut());
            }
        });
    }
}
