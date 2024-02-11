/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::platform::RenderPlatform;

pub trait BuildDrawContext<'a> {
    type DrawContext;
}

impl<'a, F, Ctx> BuildDrawContext<'a> for F
where
    F: FnOnce(&'a mut ()) -> Ctx,
{
    type DrawContext = Ctx;
}

pub type DrawContext<'a, P> = <<P as RenderPlatform>::DrawContextBuilder as BuildDrawContext<'a>>::DrawContext;
