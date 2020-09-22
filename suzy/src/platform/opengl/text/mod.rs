/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use crate::graphics::{DrawContext, Graphic};

use super::OpenGlRenderPlatform;
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

pub struct Text {
    vertices: SingleVertexBuffer<GLfloat>,
    channels: HashMap<ChannelMask, std::ops::Range<usize>>,
    texture: Texture,
}

impl Text {
    pub fn new() -> Self {
        Text {
            vertices: SingleVertexBuffer::new(true),
            channels: HashMap::new(),
            texture: Default::default(),
        }
    }

    pub fn render(
        &mut self,
        text: &str,
        font: &FontFamilyDynamic<'_>,
        settings: TextLayoutSettings,
    ) {
        self.texture = font.texture.clone();
        let mut verts = vec![];
        let channels = &mut self.channels;
        self.vertices.set_data(|_gl| {
            let mut calc = FontCharCalc::new(font, settings);
            let parser = RichTextParser::new(text);
            for rich_text_cmd in parser {
                calc.push(rich_text_cmd);
            }
            channels.clear();
            calc.merge_verts(&mut verts, channels);
            &verts[..]
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
        });
    }
}
