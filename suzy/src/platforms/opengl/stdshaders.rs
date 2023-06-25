/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::rc::Rc;

use super::{
    shader::{ProgramCompileError, ShaderProgram, UniformLoc},
    OpenGlBindings,
};

const STD_VERTEX_SOURCE: &[u8] = include_bytes!("include/std.vert.glsl");
const STD_FRAGMENT_SOURCE: &[u8] = include_bytes!("include/std.frag.glsl");

#[derive(Clone, Copy, Debug)]
pub(super) struct Uniforms {
    pub(super) transform: UniformLoc,
    pub(super) tex_id: UniformLoc,
    pub(super) tex_size: UniformLoc,
    pub(super) tex_sdf: UniformLoc,
    pub(super) tex_color_pow: UniformLoc,
}

#[derive(Clone)]
pub struct Shaders {
    pub(super) shader: ShaderProgram,
    pub(super) uniforms: Uniforms,
}

impl Shaders {
    pub fn new(gl: &Rc<OpenGlBindings>) -> Result<Self, ProgramCompileError> {
        let shader =
            ShaderProgram::create(gl, STD_VERTEX_SOURCE, STD_FRAGMENT_SOURCE)?;
        Ok(Self {
            uniforms: Uniforms {
                transform: shader.uniform(gl, "TRANSFORM"),
                tex_id: shader.uniform(gl, "TEX_ID"),
                tex_size: shader.uniform(gl, "TEX_SIZE"),
                tex_sdf: shader.uniform(gl, "TEX_SDF"),
                tex_color_pow: shader.uniform(gl, "TEX_COLOR_POW"),
            },
            shader,
        })
    }
}
