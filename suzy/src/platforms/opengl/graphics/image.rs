/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dims::{Dim, Padding2d, Rect, SimplePadding2d, SimpleRect};
use crate::graphics::{DrawContext, Graphic};
use crate::selectable::{Selectable, SelectionState, SelectionStateV2};

use crate::platforms::opengl;
use opengl::context::bindings::{FALSE, FLOAT, TRIANGLES, UNSIGNED_BYTE};
use opengl::{
    DualVertexBuffer, DualVertexBufferIndexed, OpenGlRenderPlatform, Texture,
};

#[rustfmt::skip]
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

/// A common graphic used for user interfaces, a sliced image is defined by
/// fixed-sized corners and an inner area which stretches to fill the
/// graphic area.
///
/// See the [Wikipedia article](https://en.wikipedia.org/wiki/9-slice_scaling)
/// on 9-slice scaling for more information.
pub struct SlicedImage {
    rect: SimpleRect,
    padding: SimplePadding2d,
    texture: Texture,
    buffers: DualVertexBufferIndexed<f32>,
}

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
    /// Create a new SlicedImage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the texture used by this graphic.  The given padding describes
    /// the sliced area.
    pub fn set_image<P>(&mut self, texture: Texture, padding: &P)
    where
        P: Padding2d,
    {
        self.texture = texture;
        self.padding = padding.into();
        self.update_image();
    }

    fn update_image(&mut self) {
        let mut uvs = [0f32; 32];
        let Self {
            buffers,
            texture,
            padding,
            ..
        } = self;
        buffers.set_data_1(|_gl| {
            texture.size().and_then(|(tex_width, tex_height)| {
                let uvs = &mut uvs;
                texture.transform_uvs(move || {
                    let left = padding.left() / tex_width;
                    let right = 1.0 - (padding.right() / tex_width);
                    let bottom = padding.bottom() / tex_height;
                    let top = 1.0 - (padding.top() / tex_height);
                    #[rustfmt::skip]
                    let data = [
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
                    *uvs = data;
                    &mut uvs[..]
                })
            })
        });
    }

    fn update(&mut self) {
        let mut inner = SimpleRect::default();
        inner.set_fill(&self.rect, &self.padding);
        let rect = &self.rect;
        let mut vertices = [0f32; 32];
        self.buffers.set_data_0(|_gl| {
            #[rustfmt::skip]
            let data = [
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
            vertices = data;
            &vertices[..]
        });
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
        let res = self.rect.x_mut(f);
        self.update();
        res
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R,
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
                        0,
                        2,
                        FLOAT,
                        FALSE,
                        0,
                        std::ptr::null(),
                    );
                }
                ready.bind_1();
                unsafe {
                    gl.VertexAttribPointer(
                        1,
                        2,
                        FLOAT,
                        FALSE,
                        0,
                        std::ptr::null(),
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
                self.texture.bind(ctx.render_ctx_mut());
                self.update_image();
                self.buffers.set_indices(|_gl| &SLICED_INDICES[..]);
            }
        });
    }
}

/// A version of SliceImage which assumes multiple images are layed out in
/// the same texture corosponding to different "selection states".
///
/// See the selectable module for more information on selection states.
#[derive(Default)]
pub struct SelectableSlicedImage {
    inner: SlicedImage,
    states: std::borrow::Cow<'static, [SelectionState]>,
    current_state: SelectionState,
}

impl Rect for SelectableSlicedImage {
    fn x(&self) -> Dim {
        self.inner.x()
    }
    fn y(&self) -> Dim {
        self.inner.y()
    }

    fn x_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R,
    {
        self.inner.x_mut(f)
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R,
    {
        self.inner.y_mut(f)
    }
}

impl Selectable for SelectableSlicedImage {
    fn selection_changed(&mut self, state: SelectionState) {
        self.current_state = state;
    }
}

impl SelectableSlicedImage {
    /// Create a new SelectableSlicedImage
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the texture used by this graphic.
    ///
    /// The given slice of states defines the number and varients of the
    /// sub-images in the texture.  Note: the states must contain
    /// `SelectionState::normal()` as a possible fallback for other,
    /// unincluded states.
    ///
    /// The given padding describes the sliced area of each sub-image.
    pub fn set_image<P, S>(&mut self, texture: Texture, padding: &P, states: S)
    where
        P: Padding2d,
        S: Into<std::borrow::Cow<'static, [SelectionState]>>,
    {
        self.inner.texture = texture;
        self.inner.padding = padding.into();
        let states = states.into();
        assert!(
            states.contains(&SelectionState::normal()),
            "SelectableSlicedImage must have a variant for Normal",
        );
        self.states = states;
        self.update_image();
    }

    fn update_image(&mut self) {
        const NUM_STATES: usize = 5;
        const COORDS_PER_STATE: usize = 32;
        let all_states: [_; NUM_STATES] = [
            SelectionState::normal(),
            SelectionState::hover(),
            SelectionState::focus(),
            SelectionState::pressed(),
            SelectionState::active(),
        ];
        let mut uvs = [0f32; NUM_STATES * COORDS_PER_STATE];
        let states = &self.states;
        let SlicedImage {
            buffers,
            texture,
            padding,
            ..
        } = &mut self.inner;
        buffers.set_data_1(|_gl| {
            texture.size().and_then(|(tex_width, tex_height)| {
                let uvs = &mut uvs;
                texture.transform_uvs(move || {
                    let state_frac = 1.0 / (states.len() as f32);
                    let left = padding.left() / tex_width;
                    let bottom = padding.bottom() / tex_height;
                    let top = 1.0 - (padding.top() / tex_height);
                    for (i, state) in all_states.iter().enumerate() {
                        let data_start = i * COORDS_PER_STATE;
                        let data_end = data_start + COORDS_PER_STATE;
                        let mut find_state = *state;
                        let state_index: usize = loop {
                            if let Some(idx) = states
                                .iter()
                                .position(|item| *item == find_state)
                            {
                                break idx;
                            }
                            find_state = find_state.reduce();
                        };
                        let offset = (state_index as f32) * state_frac;
                        let left = offset + left;
                        let end = offset + state_frac;
                        let right = end - (padding.right() / tex_width);
                        #[rustfmt::skip]
                        uvs[data_start..data_end].copy_from_slice(&[
                            offset, 0.0,
                            end, 0.0,
                            end, 1.0,
                            offset, 1.0,
                            left, 0.0,
                            right, 0.0,
                            end, bottom,
                            end, top,
                            right, 1.0,
                            left, 1.0,
                            offset, top,
                            offset, bottom,
                            left, bottom,
                            right, bottom,
                            right, top,
                            left, top,
                        ]);
                    }
                    &mut uvs[..]
                })
            })
        });
    }
}

impl Graphic<OpenGlRenderPlatform> for SelectableSlicedImage {
    fn draw(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>) {
        const UV_STATE_SIZE: usize = 32 * std::mem::size_of::<f32>();
        let uv_offset = match self.current_state.v2() {
            SelectionStateV2::Normal => 0,
            SelectionStateV2::Hover => UV_STATE_SIZE,
            SelectionStateV2::Focus => UV_STATE_SIZE * 2,
            SelectionStateV2::Pressed => UV_STATE_SIZE * 3,
            SelectionStateV2::Active => UV_STATE_SIZE * 4,
        };
        ctx.push(|ctx| {
            ctx.params().standard_mode();
            ctx.params().use_texture(self.inner.texture.clone());
            if let Some(ready) = self.inner.buffers.check_ready(ctx) {
                let gl = ready.gl;
                ready.bind_0();
                unsafe {
                    gl.VertexAttribPointer(
                        0,
                        2,
                        FLOAT,
                        FALSE,
                        0,
                        std::ptr::null(),
                    );
                }
                ready.bind_1();
                unsafe {
                    gl.VertexAttribPointer(
                        1,
                        2,
                        FLOAT,
                        FALSE,
                        0,
                        uv_offset as _,
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
                self.inner.update();
                self.inner.texture.bind(ctx.render_ctx_mut());
                self.update_image();
                self.inner.buffers.set_indices(|_gl| &SLICED_INDICES[..]);
            }
        });
    }
}

/// An image which will stretch the texture to fill the image rect.
pub struct SimpleImage {
    rect: SimpleRect,
    texture: Texture,
    buffers: DualVertexBuffer<f32>,
}

impl Default for SimpleImage {
    fn default() -> Self {
        Self {
            rect: SimpleRect::default(),
            texture: Texture::default(),
            buffers: DualVertexBuffer::new(true, false),
        }
    }
}

impl SimpleImage {
    /// Create a new SimpleImage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the texture used by this graphic.
    pub fn set_image(&mut self, texture: Texture) {
        self.texture = texture;
        self.update_image();
    }

    fn update_image(&mut self) {
        let mut uvs = [0f32; 12];
        let Self {
            buffers, texture, ..
        } = self;
        buffers.set_data_1(|_gl| {
            texture.size().and_then(|_| {
                let uvs = &mut uvs;
                texture.transform_uvs(move || {
                    #[rustfmt::skip]
                    let data = [
                        0.0, 0.0,
                        1.0, 0.0,
                        1.0, 1.0,
                        0.0, 0.0,
                        1.0, 1.0,
                        0.0, 1.0,
                    ];
                    *uvs = data;
                    &mut uvs[..]
                })
            })
        });
    }

    fn update(&mut self) {
        let rect = &self.rect;
        let mut vertices = [0f32; 12];
        self.buffers.set_data_0(|_gl| {
            #[rustfmt::skip]
            let data = [
                rect.left(), rect.bottom(),
                rect.right(), rect.bottom(),
                rect.right(), rect.top(),
                rect.left(), rect.bottom(),
                rect.right(), rect.top(),
                rect.left(), rect.top(),
            ];
            vertices = data;
            &vertices[..]
        });
    }
}

impl Rect for SimpleImage {
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
        let res = self.rect.x_mut(f);
        self.update();
        res
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R,
    {
        let res = self.rect.y_mut(f);
        self.update();
        res
    }
}

impl Graphic<OpenGlRenderPlatform> for SimpleImage {
    fn draw(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>) {
        ctx.push(|ctx| {
            ctx.params().standard_mode();
            ctx.params().use_texture(self.texture.clone());
            if let Some(ready) = self.buffers.check_ready(ctx) {
                let gl = ready.gl;
                ready.bind_0();
                unsafe {
                    gl.VertexAttribPointer(
                        0,
                        2,
                        FLOAT,
                        FALSE,
                        0,
                        std::ptr::null(),
                    );
                }
                ready.bind_1();
                unsafe {
                    gl.VertexAttribPointer(
                        1,
                        2,
                        FLOAT,
                        FALSE,
                        0,
                        std::ptr::null(),
                    );
                }
                unsafe {
                    gl.DrawArrays(TRIANGLES, 0, 6);
                }
            } else {
                self.update();
                self.texture.bind(ctx.render_ctx_mut());
                self.update_image();
            }
        });
    }
}
