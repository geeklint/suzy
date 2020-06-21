use std::collections::HashMap;

use super::OpenGlRenderPlatform as Gl;
use super::bindings::types::*;
use super::bindings::{
    ARRAY_BUFFER,
    FALSE,
    FLOAT,
    TRIANGLES,
};
use super::sdf::SdfRenderPlatform;
use super::primitive::{
    Texture,
    Buffer,
};

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
    vertices: Buffer<GLfloat>,
    channels: HashMap<ChannelMask, std::ops::Range<usize>>,
    texture: Texture,
}

impl Text {
    pub fn new() -> Self {
        Text {
            vertices: Buffer::new(ARRAY_BUFFER, true, 0),
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
        let mut calc = FontCharCalc::new(font, settings);
        let parser = RichTextParser::new(text);
        for rich_text_cmd in parser {
            calc.push(rich_text_cmd);
        }
        self.channels.clear();
        calc.merge_verts(&mut verts, &mut self.channels);
        self.vertices.set_data(&verts);
    }
}

use crate::graphics::{DrawContext, Graphic};

impl Graphic<Gl> for Text {
    fn draw(&self, ctx: &mut DrawContext<Gl>) {
        ctx.descend(|ctx| Graphic::<SdfRenderPlatform>::draw(self, ctx));
    }
}

impl Graphic<SdfRenderPlatform> for Text {
    fn draw(&self, ctx: &mut DrawContext<SdfRenderPlatform>) {
        let mut params = ctx.clone_current();
        params.use_texture(self.texture.clone());
        DrawContext::push(ctx, params.clone());
        Gl::global(|gl| unsafe {
            self.vertices.bind();
            let stride = (4 * std::mem::size_of::<GLfloat>()) as _;
            let offset = (2 * std::mem::size_of::<GLfloat>()) as _;
            gl.VertexAttribPointer(
                0,
                2,
                FLOAT,
                FALSE,
                stride,
                std::ptr::null(),
            );
            gl.VertexAttribPointer(
                1,
                2,
                FLOAT,
                FALSE,
                stride,
                offset,
            );
            for (mask, range) in self.channels.iter() {
                let mut masked_params = params.clone();
                masked_params.use_tex_chan_mask(*mask);
                DrawContext::push(ctx, masked_params);
                gl.DrawArrays(
                    TRIANGLES,
                    range.start as GLsizei,
                    range.len() as GLsizei,
                );
                DrawContext::pop(ctx);
            }
        });
        DrawContext::pop(ctx);
    }
}
