/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2023 Violet Leonard */

use crate::{
    graphics::{DrawContext, Graphic},
    platforms::opengl,
    widget::WidgetGraphic,
};

use opengl::{Mat4, OpenGlRenderPlatform};

#[derive(Clone, Copy, Debug, Default)]
pub struct Transform {
    pub matrix: Mat4,
    original: Mat4,
}

pub struct Push<'a> {
    trans: &'a mut Transform,
}

pub struct Pop<'a> {
    trans: &'a Transform,
}

impl WidgetGraphic<OpenGlRenderPlatform> for Transform {
    type BeforeGetter = fn(&mut ()) -> Push<'_>;
    type AfterGetter = fn(&mut ()) -> Pop<'_>;

    fn before_children(&mut self) -> Push<'_> {
        Push { trans: self }
    }

    fn after_children(&mut self) -> Pop<'_> {
        Pop { trans: self }
    }
}

impl Graphic<OpenGlRenderPlatform> for Push<'_> {
    fn draw(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {
        ctx.update_matrix(|mat| {
            self.trans.original = mat;
            mat * self.trans.matrix
        });
    }
}

impl Graphic<OpenGlRenderPlatform> for Pop<'_> {
    fn draw(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {
        ctx.update_matrix(|_| self.trans.original);
    }
}
