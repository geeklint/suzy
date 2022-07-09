/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::platform::RenderPlatform;

pub trait PlatformDrawContext<NextPass> {
    fn finish(self) -> Option<NextPass>;
}

pub trait BuildDrawContext<'a, NextPass> {
    type DrawContext: PlatformDrawContext<NextPass>;
}

impl<'a, F, Ctx, NextPass> BuildDrawContext<'a, NextPass> for F
where
    F: FnOnce(&'a mut ()) -> Ctx,
    Ctx: PlatformDrawContext<NextPass>,
{
    type DrawContext = Ctx;
}

pub type DrawContext<'a, P> =
    <<P as RenderPlatform>::DrawContextBuilder as BuildDrawContext<
        'a,
        <P as RenderPlatform>::DrawPassInfo,
    >>::DrawContext;
