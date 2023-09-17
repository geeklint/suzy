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

pub struct TransformPush<'a> {
    trans: &'a mut Transform,
}

pub struct TransformPop<'a> {
    trans: &'a Transform,
}

impl WidgetGraphic<OpenGlRenderPlatform> for Transform {
    type BeforeGetter = fn(&mut ()) -> TransformPush<'_>;
    type AfterGetter = fn(&mut ()) -> TransformPop<'_>;

    fn before_children(&mut self) -> TransformPush<'_> {
        TransformPush { trans: self }
    }

    fn after_children(&mut self) -> TransformPop<'_> {
        TransformPop { trans: self }
    }
}

impl Graphic<OpenGlRenderPlatform> for TransformPush<'_> {
    fn draw(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {
        ctx.update_matrix(|mat| {
            self.trans.original = mat;
            mat * self.trans.matrix
        })
    }
}

impl Graphic<OpenGlRenderPlatform> for TransformPop<'_> {
    fn draw(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {
        ctx.update_matrix(|_| self.trans.original)
    }
}
