/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! Types for dealing with formatted text.

/// An enum describing horizontal text alignment settings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Alignment {
    /// Left-aligned
    #[default]
    Left,
    /// Center-aligned
    Center,
    /// Right-aligned
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Line {
    #[default]
    Ascent,
    Descent,
    Baseline,
    BetweenBaseAndCap,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Flow {
    #[default]
    Down,
    Up,
    Out,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum VerticalLimit {
    #[default]
    None,
    Lines(u16),
    Height(f32),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverflowMode {
    #[default]
    Truncate,
    Ellipsis,
}

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub alignment: Alignment,
    pub line: Line,
    pub flow: Flow,
    pub origin_x: f32,
    pub origin_y: f32,
    pub wrap_width: f32,
    pub vertical_limit: VerticalLimit,
    pub overflow_mode: OverflowMode,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            alignment: Alignment::Left,
            line: Line::Ascent,
            flow: Flow::Down,
            origin_x: 0.0,
            origin_y: 100.0,
            wrap_width: f32::INFINITY,
            vertical_limit: VerticalLimit::None,
            overflow_mode: OverflowMode::Truncate,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RichTextTagParseError {
    pub msg: String,
}
