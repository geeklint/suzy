/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::rc::Rc;

use super::shader::{ProgramCompileError, Shader, UniformLoc};
use super::OpenGlBindings;

const STD_VERTEX_SOURCE: &[u8] = include_bytes!("std.vert.glsl");
const STD_FRAGMENT_SOURCE: &[u8] = include_bytes!("std.frag.glsl");

#[derive(Clone, Copy, Debug)]
pub(super) struct Uniforms {
    pub(super) transform: UniformLoc,
    pub(super) tex_id: UniformLoc,
    pub(super) tint_color: UniformLoc,
    pub(super) mask_id: UniformLoc,
    pub(super) mask_bounds: UniformLoc,
    pub(super) sdf_values: UniformLoc,
    pub(super) sdf_chan_mask: UniformLoc,
}

#[derive(Clone)]
pub struct Shaders {
    pub(super) shader: Shader,
    pub(super) uniforms: Uniforms,
}

impl Shaders {
    pub fn new(gl: &Rc<OpenGlBindings>) -> Result<Self, ProgramCompileError> {
        let shader =
            Shader::create(gl, STD_VERTEX_SOURCE, STD_FRAGMENT_SOURCE)?;
        Ok(Self {
            uniforms: Uniforms {
                transform: shader.uniform(gl, "TRANSFORM"),
                tex_id: shader.uniform(gl, "TEX_ID"),
                tint_color: shader.uniform(gl, "TINT_COLOR"),
                mask_id: shader.uniform(gl, "MASK_ID"),
                mask_bounds: shader.uniform(gl, "MASK_BOUNDS"),
                sdf_values: shader.uniform(gl, "SDF_VALUES"),
                sdf_chan_mask: shader.uniform(gl, "SDF_CHAN_MASK"),
            },
            shader,
        })
    }
}
