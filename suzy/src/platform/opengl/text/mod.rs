/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use crate::watch::{Watched};
use crate::graphics::{Color, DrawContext, Graphic};

use super::OpenGlRenderPlatform;
use super::Mat4;
use super::context::bindings::types::*;
use super::context::bindings::{
    FALSE,
    FLOAT,
    TRIANGLES,
};
use super::texture::Texture;
use super::buffer::SingleVertexBuffer;

mod font;
mod calc;

pub use font::{
    FontFamily,
    FontFamilySource,
    FontFamilyDynamic,
    FontFamilySourceDynamic,
};

pub use calc::{
    FontStyle,
    TextAlignment,
    TextLayoutSettings,
    RichTextCommand,
    RichTextParser,
};

use font::{ChannelMask, GlyphMetricsSource};
use calc::FontCharCalc;

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


pub struct Text {
    vertices: SingleVertexBuffer<GLfloat>,
    channels: HashMap<ChannelMask, std::ops::Range<usize>>,
    texture: Texture,
    font: Watched<Option<Box<FontFamily>>>,
    render_settings: TextRenderSettings,
}

impl Text {
    pub fn new() -> Self {
        Text {
            vertices: SingleVertexBuffer::new(true),
            channels: HashMap::new(),
            texture: Default::default(),
            font: Watched::new(None),
            render_settings: TextRenderSettings::default(),
        }
    }

    pub fn set_text(&mut self, text: &str, settings: TextLayoutSettings) {
        let texture = &mut self.texture;
        let vertices = &mut self.vertices;
        let channels = &mut self.channels;
        let mut do_render = move |font: &FontFamilyDynamic| {
            *texture = font.texture.clone();
            Self::render_impl(
                text,
                settings,
                vertices,
                channels,
                font,
            )
        };
        match &*self.font {
            Some(font) => do_render(font),
            None => with_default_font(do_render),
        };
    }

    pub fn set_font(&mut self, font: Box<FontFamily>) {
        *self.font = Some(font);
    }

    pub fn render_settings(&mut self) -> &mut TextRenderSettings {
        &mut self.render_settings
    }

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
        )
    }

    fn render_impl(
        text: &str,
        settings: TextLayoutSettings,
        vertices: &mut SingleVertexBuffer<GLfloat>,
        channels: &mut HashMap<ChannelMask, std::ops::Range<usize>>,
        font: &FontFamilyDynamic,
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
            ctx.params().outline_color(self.render_settings.outline_color);
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
                        0, 2, FLOAT, FALSE, stride, std::ptr::null(),
                    );
                    gl.VertexAttribPointer(
                        1, 2, FLOAT, FALSE, stride, offset,
                    );
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

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct TextRenderSettings {
    pub text_color: Color,
    pub outline_color: Color,
    pub pseudo_bold_level: f32,
    pub outline_width: f32,
    pub smoothing: f32,
    pub outline_smoothing: f32,
    pub x: f32,
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
