/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use suzy::graphics::Color;

// use this to ensure we split at pixel boundries (4 bytes)
#[allow(unused)]
pub const ALIGN_MASK: usize = usize::MAX ^ 3;

// screen scaling might cause edges to be antialiased, and not exactly
// white or black, use these rounding functions to shrink the area we check
#[must_use]
#[allow(unused)]
pub fn round_front(buffer: &[u8]) -> &[u8] {
    let start = (buffer.len() / 20) & ALIGN_MASK;
    &buffer[start..]
}

#[must_use]
#[allow(unused)]
pub fn round_back(buffer: &[u8]) -> &[u8] {
    &buffer[..(buffer.len() * 19 / 20) & ALIGN_MASK]
}

#[must_use]
#[allow(unused)]
pub fn round_both(buffer: &[u8]) -> &[u8] {
    let start = (buffer.len() / 20) & ALIGN_MASK;
    let end = (buffer.len() * 19 / 20) & ALIGN_MASK;
    &buffer[start..end]
}

#[must_use]
#[allow(unused)]
pub fn is_color(buffer: &[u8], color: Color) -> bool {
    buffer.chunks_exact(4).all(|chunk| {
        let buf_color =
            Color::from_rgba8(chunk[0], chunk[1], chunk[2], chunk[3]);
        buf_color == color
    })
}
