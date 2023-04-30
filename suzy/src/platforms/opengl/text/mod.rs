/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::collections::HashMap;

use crate::{
    graphics::{Color, DrawContext, Graphic},
    text,
    watch::Watched,
};

use super::{
    buffer::SingleVertexBuffer,
    context::bindings::types::*,
    context::bindings::{FALSE, FLOAT, TRIANGLES},
    texture::Texture,
    Mat4, OpenGlRenderPlatform,
};

mod calc;
mod font;

pub use font::{
    FontFamily, FontFamilyDynamic, FontFamilySource, FontFamilySourceDynamic,
};

use calc::{CharLocationError, CharLocationRecorder, FontCharCalc};
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

pub struct TextStyle {
    pub font_size: f32,
    pub color: Color,
}

impl crate::platform::graphics::TextStyle for TextStyle {
    fn with_size_and_color(size: f32, color: Color) -> Self {
        Self {
            font_size: size,
            color,
        }
    }

    fn push_tag(&self, tag: &mut &str) -> Result<Self, ()> {
        Err(())
    }
}

/// Default Graphic for displaying Text.
///
/// This implementation is based on a signed distance field font atlas, these
/// fonts can be generated using the crate `suzy_build_tools`.
#[derive(Default)]
pub struct Text {
    raw: RawText<()>,
    font: Watched<Option<Box<FontFamily>>>,
    render_settings: TextRenderSettings,
    layout: text::Layout,
}

impl Text {
    /// Set the font to be used by future calls to `set_text`.
    pub fn set_font(&mut self, font: Box<FontFamily>) {
        *self.font = Some(font);
    }
}

impl Graphic<OpenGlRenderPlatform> for Text {
    fn draw(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {
        self.raw.draw(ctx, self.render_settings);
    }
}

impl crate::platform::graphics::Text<TextStyle> for Text {
    fn set_layout(&mut self, layout: text::Layout) {
        self.layout = layout;
        self.render_settings.x = layout.origin_x;
        self.render_settings.y = layout.origin_y;
    }

    fn clear(&mut self) {
        self.raw.channels.clear();
    }

    fn push_span(&mut self, style: TextStyle, text: &str) {
        self.render_settings.text_color = style.color;
        match &*self.font {
            Some(font) => {
                self.raw.render(text, font, self.layout, style.font_size)
            }
            None => with_default_font(|font| {
                self.raw.render(text, font, self.layout, style.font_size);
            }),
        };
    }
}

/// RawText for custom use cases or optimization
///
/// Text is done in two stages, and there are two settings types for each
/// stage:
///
/// 1. `TextLayoutSettings` controls the generation of the text vertices, and
/// contains settings like alignment and wrap width.
/// 2. `TextRenderSettings` controls the rendering of the text, and contains
/// settings such as text color and position.
pub struct RawText<T> {
    vertices: SingleVertexBuffer<GLfloat>,
    channels: HashMap<ChannelMask, std::ops::Range<usize>>,
    texture: Texture,
    char_locs: T,
}

impl<T: Default> Default for RawText<T> {
    fn default() -> Self {
        Self {
            vertices: SingleVertexBuffer::new(true),
            channels: HashMap::new(),
            texture: Texture::default(),
            char_locs: T::default(),
        }
    }
}

impl<T: Default> RawText<T> {
    /// Create a new empty text graphic.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: calc::RecordCharLocation> RawText<T> {
    /// Update the text to display, using the provided font.
    pub fn render(
        &mut self,
        text: &str,
        font: &FontFamilyDynamic<'_>,
        settings: text::Layout,
        font_size: f32,
    ) {
        self.texture = font.texture.clone();
        let vertices = &mut self.vertices;
        let channels = &mut self.channels;
        let char_locs = &mut self.char_locs;
        let mut verts = vec![];
        vertices.set_data(|_gl| {
            font.texture.transform_uvs(|| {
                let mut calc = FontCharCalc::new(font, settings, char_locs);
                calc.push_str(font_size, text);
                channels.clear();
                calc.merge_verts(&mut verts, channels);
                &mut verts[..]
            })
        });
    }
}

impl RawText<CharLocationRecorder> {
    /// Get the index of the character at the given position
    pub fn char_at(&self, x: f32, y: f32) -> Result<usize, CharLocationError> {
        self.char_locs.char_at(x, y)
    }

    /// Get the bounding rect of the character at the given index
    fn char_rect(&self, index: usize) -> Option<crate::dims::SimpleRect> {
        self.char_locs.char_rect(index)
    }
}

impl<T> RawText<T> {
    fn draw(
        &mut self,
        ctx: &mut DrawContext<'_, OpenGlRenderPlatform>,
        render_settings: TextRenderSettings,
    ) {
        ctx.push(|ctx| {
            ctx.params().sdf_mode();
            ctx.params().use_texture(self.texture.clone());
            ctx.params().transform(Mat4::translate(
                render_settings.x,
                render_settings.y,
            ));
            let opt_text = Some((
                render_settings.text_color,
                render_settings.pseudo_bold_level,
                render_settings.smoothing,
            ));
            let opt_outline = (render_settings.outline_width
                > render_settings.pseudo_bold_level)
                .then(|| {
                    (
                        render_settings.outline_color,
                        render_settings.outline_width,
                        render_settings.outline_smoothing,
                    )
                });
            let opt_shadow = None;
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
                for (color, edge, smoothing) in
                    opt_shadow.into_iter().chain(opt_outline).chain(opt_text)
                {
                    ctx.push(|ctx| {
                        ctx.params().tint(color);
                        ctx.params().sdf_edge(edge, smoothing);
                        for (mask, range) in self.channels.iter() {
                            #[allow(clippy::len_zero)]
                            if range.len() > 0 {
                                ctx.push(|ctx| {
                                    ctx.params().sdf_chan_mask(*mask);
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
                    });
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
