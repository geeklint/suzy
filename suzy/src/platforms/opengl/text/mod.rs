/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use crate::graphics::{Color, DrawContext, Graphic};
use crate::text::{FontStyle, RichTextParser};
use crate::watch::Watched;

use super::buffer::SingleVertexBuffer;
use super::context::bindings::types::*;
use super::context::bindings::{FALSE, FLOAT, TRIANGLES};
use super::texture::Texture;
use super::Mat4;
use super::OpenGlRenderPlatform;

mod calc;
mod font;

pub use calc::TextLayoutSettings;
pub use font::{
    FontFamily, FontFamilyDynamic, FontFamilySource, FontFamilySourceDynamic,
};

use calc::FontCharCalc;
use font::{ChannelMask, GlyphMetricsSource};

#[cfg(feature = "default_font")]
mod default_font;

#[cfg(feature = "default_font")]
thread_local! {
    static DEFAULT_FONT: FontFamily = default_font::FONT.load();
}

#[cfg(feature = "default_font")]
fn with_default_font<F: FnOnce(&FontFamily) -> R, R>(f: F) -> R {
    DEFAULT_FONT.with(f)
}

#[cfg(not(feature = "default_font"))]
#[track_caller]
fn with_default_font<F: FnOnce(&FontFamily) -> R, R>(_f: F) -> R {
    panic!(concat!(
        "`Text::set_text` called without an available font.",
        " Perhaps you need to call `Text::set_font` first, or",
        " enable the feature `default_font`.",
    ));
}

/// Default Graphic for displaying Text.
///
/// This implementation is based on a signed distance field font atlas, these
/// fonts can be generated using the crate `suzy_build_tools`.
///
/// Text is done in two stages, and there are two settings types for each
/// stage:
///
/// 1. `TextLayoutSettings` controls the generation of the text vertices, and
/// contains settings like alignment and wrap width.
/// 2. `TextRenderSettings` controls the rendering of the text, and contains
/// settings such as text color and position.
pub struct Text {
    vertices: SingleVertexBuffer<GLfloat>,
    channels: HashMap<ChannelMask, std::ops::Range<usize>>,
    texture: Texture,
    font: Watched<Option<Box<FontFamily>>>,
    render_settings: TextRenderSettings,
    char_locs: Vec<(f32, f32)>,
}

impl Text {
    /// Create a new empty text graphic.
    pub fn new() -> Self {
        Text {
            vertices: SingleVertexBuffer::new(true),
            channels: HashMap::new(),
            texture: Texture::default(),
            font: Watched::new(None),
            render_settings: TextRenderSettings::default(),
            char_locs: Vec::new(),
        }
    }

    pub fn char_at(&self, x: f32, y: f32) -> Result<usize, ()> {
        let mut err = false;
        let search_res = self.char_locs.binary_search_by(|(cx, cy)| {
            println!("{:?}", (cx, cy));
            if y <= *cy && y > *cy - 24.0 {
                cx.partial_cmp(&x).unwrap_or_else(|| {
                    err = true;
                    std::cmp::Ordering::Equal
                })
            } else {
                cy.partial_cmp(&y)
                    .unwrap_or_else(|| {
                        err = true;
                        std::cmp::Ordering::Equal
                    })
                    .reverse()
            }
        });
        if err {
            return Err(());
        }
        match search_res {
            Ok(index) => Ok(index),
            Err(index) => Ok(index - 1),
        }
    }

    /// Update the text to display, using the current font, or a default font.
    ///
    /// Panics if a font has not been assigned using set_font, and the
    /// default_font feature has been disabled.
    pub fn set_text(&mut self, text: &str, settings: TextLayoutSettings) {
        let texture = &mut self.texture;
        let vertices = &mut self.vertices;
        let channels = &mut self.channels;
        let char_locs = &mut self.char_locs;
        let mut do_render = move |font: &FontFamilyDynamic| {
            *texture = font.texture.clone();
            Self::render_impl(
                text, settings, vertices, channels, font, char_locs,
            )
        };
        match &*self.font {
            Some(font) => do_render(font),
            None => with_default_font(do_render),
        };
    }

    /// Set the font to be used by future calls to `set_text`.
    pub fn set_font(&mut self, font: Box<FontFamily>) {
        *self.font = Some(font);
    }

    /// Get a mutable reference to this graphics render settings, so you can
    /// change attributes like the text color.
    pub fn render_settings(&mut self) -> &mut TextRenderSettings {
        &mut self.render_settings
    }

    /// Update the text to display, using the provided font.
    pub fn render(
        &mut self,
        text: &str,
        font: &FontFamilyDynamic,
        settings: TextLayoutSettings,
    ) {
        self.texture = font.texture.clone();
        Self::render_impl(
            text,
            settings,
            &mut self.vertices,
            &mut self.channels,
            font,
            &mut self.char_locs,
        )
    }

    fn render_impl(
        text: &str,
        settings: TextLayoutSettings,
        vertices: &mut SingleVertexBuffer<GLfloat>,
        channels: &mut HashMap<ChannelMask, std::ops::Range<usize>>,
        font: &FontFamilyDynamic,
        char_locs: &mut Vec<(f32, f32)>,
    ) {
        let mut verts = vec![];
        vertices.set_data(|_gl| {
            font.texture.transform_uvs(|| {
                let mut calc = FontCharCalc::new(font, settings);
                let parser = RichTextParser::new(text);
                for rich_text_cmd in parser {
                    calc.push(rich_text_cmd);
                }
                channels.clear();
                calc.merge_verts(&mut verts, channels);
                *char_locs = calc.take_char_locs();
                &mut verts[..]
            })
        });
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}

impl Graphic<OpenGlRenderPlatform> for Text {
    fn draw(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>) {
        ctx.push(|ctx| {
            ctx.params().sdf_mode();
            ctx.params().use_texture(self.texture.clone());
            ctx.params().text_color(self.render_settings.text_color);
            ctx.params()
                .outline_color(self.render_settings.outline_color);
            ctx.params().body_edge(
                self.render_settings.pseudo_bold_level,
                self.render_settings.smoothing,
            );
            ctx.params().outline_edge(
                self.render_settings.outline_width,
                self.render_settings.outline_smoothing,
            );
            ctx.params().transform(Mat4::translate(
                self.render_settings.x,
                self.render_settings.y,
            ));
            if self.vertices.bind_if_ready(ctx) {
                let stride = (4 * std::mem::size_of::<GLfloat>()) as _;
                let offset = (2 * std::mem::size_of::<GLfloat>()) as _;
                let gl = &ctx.render_ctx().bindings;
                unsafe {
                    gl.VertexAttribPointer(
                        0,
                        2,
                        FLOAT,
                        FALSE,
                        stride,
                        std::ptr::null(),
                    );
                    gl.VertexAttribPointer(1, 2, FLOAT, FALSE, stride, offset);
                }
                for (mask, range) in self.channels.iter() {
                    #[allow(clippy::len_zero)]
                    if range.len() > 0 {
                        ctx.push(|ctx| {
                            ctx.params().tex_chan_mask(*mask);
                            ctx.prepare_draw();
                            let gl = &ctx.render_ctx().bindings;
                            unsafe {
                                gl.DrawArrays(
                                    TRIANGLES,
                                    range.start as GLsizei,
                                    range.len() as GLsizei,
                                );
                            }
                        });
                    }
                }
            } else {
                self.texture.bind(ctx.render_ctx_mut());
            }
        });
    }
}

/// Settings controlling the rendering of the text.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct TextRenderSettings {
    /// Primary text color.
    pub text_color: Color,

    /// Color of outline around glyphs.
    pub outline_color: Color,

    /// Allows artificially bolding or lightening the text.
    ///
    /// (As opposed to using a different font which was designed to be bolder).
    /// The default is 0.5, which indicates no change.
    pub pseudo_bold_level: f32,

    /// Width of the outline around glyphs.
    pub outline_width: f32,

    /// Smoothing value applied to the edge of the glyph.
    ///
    /// Smaller values will cause text to appear more pixelated, larger
    /// values may cause it to appear blurry.
    pub smoothing: f32,

    /// Smoothing value applied to the edge of the outline around the glyph.
    ///
    /// Smaller values will cause text to appear more pixelated, larger
    /// values may cause it to appear blurry.
    pub outline_smoothing: f32,

    /// x position of the rendered text.
    pub x: f32,

    /// y position of the rendered text.
    pub y: f32,
}

impl Default for TextRenderSettings {
    fn default() -> Self {
        TextRenderSettings {
            text_color: Color::WHITE,
            outline_color: Color::create_rgba(1.0, 1.0, 1.0, 0.0),
            pseudo_bold_level: 0.5,
            outline_width: 0.0,
            smoothing: 0.07,
            outline_smoothing: 0.07,
            x: 0.0,
            y: 0.0,
        }
    }
}
