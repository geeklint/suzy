use std::ffi::c_void;
use std::rc::Rc;

use gl::types::*;

use crate::graphics::{self, DrawContext};
use crate::dims::{Dim, Rect, SimpleRect, SimplePadding2d, Padding2d};
use crate::math::Color;
use super::graphic::Graphic;
pub use super::primitive::{
    Texture, TextureLoader, TextureLoadResult, TextureBuilder
};

#[derive(Clone)]
pub struct SlicedImage {
    rect: SimpleRect,
    padding: SimplePadding2d,
    graphic: Graphic,
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

impl SlicedImage {
    pub fn create<R, P>(texture: Texture, rect: R, padding: P) -> Self
    where
        R: Into<SimpleRect>,
        P: Into<SimplePadding2d>,
    {
        let padding = padding.into();
        let mut graphic = Graphic::new(16, 18, (true, false, false));
        let coords = [0.0; 32];
        graphic.set_coords(&coords);
        Self::calc_uvs(&mut graphic, &padding, &texture);
        graphic.set_tris(&SLICED_INDICES);
        graphic.set_texture(texture);
        SlicedImage {
            rect: rect.into(),
            padding,
            graphic,
        }
    }

    fn calc_uvs<P>(graphic: &mut Graphic, padding: &P, texture: &Texture)
        where P: Padding2d
    {
        let left = padding.left() / texture.width();
        let right = 1.0 - (padding.right() / texture.width());
        let bottom = padding.bottom() / texture.height();
        let top = 1.0 - (padding.top() / texture.height());
        let uvs: [GLfloat; 32] = [
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
        graphic.set_uvs(&uvs);
    }

    fn update(&mut self) {
        let mut inner = SimpleRect::default();
        inner.set_fill(&self.rect, &self.padding);
        let vertices: [GLfloat; 32] = [
            // outer corners
            self.rect.left(), self.rect.bottom(),
            self.rect.right(), self.rect.bottom(),
            self.rect.right(), self.rect.top(),
            self.rect.left(), self.rect.top(),
            // bottom edge
            inner.left(), self.rect.bottom(),
            inner.right(), self.rect.bottom(),
            // right edge
            self.rect.right(), inner.bottom(),
            self.rect.right(), inner.top(),
            // top edge
            inner.right(), self.rect.top(),
            inner.left(), self.rect.top(),
            // left edge
            self.rect.left(), inner.top(),
            self.rect.left(), inner.bottom(),
            // inner corners
            inner.left(), inner.bottom(),
            inner.right(), inner.bottom(),
            inner.right(), inner.top(),
            inner.left(), inner.top(),
        ];
        self.graphic.set_coords(&vertices);
    }
}

impl Rect for SlicedImage {
    fn x(&self) -> Dim { self.rect.x() }
    fn y(&self) -> Dim { self.rect.y() }

    fn x_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) {
        self.rect.x_mut(f);
        self.update();
    }

    fn y_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) {
        self.rect.y_mut(f);
        self.update();
    }
}

impl graphics::Graphic for SlicedImage {
    fn draw(&self, ctx: &mut DrawContext) {
        self.graphic.draw(ctx);
    }
}
