/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::rc::Rc;

use super::shader::{ProgramCompileError, ShaderProgram, UniformLoc};
use super::OpenGlBindings;

const STD_VERTEX_SOURCE: &[u8] = include_bytes!("std.vert.glsl");
const STD_FRAGMENT_SOURCE: &[u8] = include_bytes!("std.frag.glsl");

#[derive(Clone, Copy, Debug)]
pub(super) struct Uniforms {
    pub(super) transform: UniformLoc,
    pub(super) tex_id: UniformLoc,
    pub(super) tex_size: UniformLoc,
    pub(super) tex_sdf: UniformLoc,
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
            },
            shader,
        })
    }
}
