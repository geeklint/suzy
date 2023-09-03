/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use super::WidgetRect;

pub(super) struct WidgetInternal<T>
where
    T: ?Sized,
{
    pub(super) rect: WidgetRect,
    pub(super) content: T,
}

impl<T> Default for WidgetInternal<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            rect: WidgetRect::default(),
            content: T::default(),
        }
    }
}
