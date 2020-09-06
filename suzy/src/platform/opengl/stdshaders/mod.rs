/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use super::OpenGlBindings;
use super::shader::{
    ProgramCompileError,
    Shader,
    UniformLoc,
};

const STD_VERTEX_SOURCE: &[u8] = include_bytes!(
    "std.vert.glsl");
const STD_FRAGMENT_SOURCE: &[u8] = include_bytes!(
    "std.frag.glsl");
const SDF_FRAGMENT_SOURCE: &[u8] = include_bytes!(
    "sdf.frag.glsl");

#[derive(Clone, Copy, Debug)]
pub(super) struct SharedUniforms {
    pub(super) transform: UniformLoc,
    pub(super) tex_transform: UniformLoc,
    pub(super) mask_id: UniformLoc,
    pub(super) mask_bounds: UniformLoc,
}

fn common(gl: &OpenGlBindings, shader: &Shader) -> SharedUniforms {
    SharedUniforms {
        transform: shader.uniform(gl, "TRANSFORM"),
        tex_transform: shader.uniform(gl, "TEX_TRANSFORM"),
        mask_id: shader.uniform(gl, "MASK_ID"),
        mask_bounds: shader.uniform(gl, "MASK_BOUNDS"),
    }
}
    
#[derive(Clone, Copy, Debug)]
pub(super) struct StdUniforms {
    pub(super) common: SharedUniforms,
    pub(super) tex_id: UniformLoc,
    pub(super) tint_color: UniformLoc,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SdfUniforms {
    pub(super) common: SharedUniforms,
    pub(super) text_color: UniformLoc,
    pub(super) outline_color: UniformLoc,
    pub(super) distance_edges: UniformLoc,
    pub(super) tex_id: UniformLoc,
    pub(super) tex_chan_mask: UniformLoc,
}

#[derive(Clone)]
pub struct Shaders {
    pub(super) std: Shader,
    pub(super) sdf: Shader,
    pub(super) std_uniforms: StdUniforms,
    pub(super) sdf_uniforms: SdfUniforms,
}

impl Shaders {
    pub fn new(gl: &Rc<OpenGlBindings>) -> Result<Self, ProgramCompileError> {
        let std = Shader::create(gl, STD_VERTEX_SOURCE, STD_FRAGMENT_SOURCE)?;
        let sdf = Shader::create(gl, STD_VERTEX_SOURCE, SDF_FRAGMENT_SOURCE)?;
        Ok(Self {
            std_uniforms: StdUniforms {
                common: common(gl, &std),
                tex_id: std.uniform(gl, "TEX_ID"),
                tint_color: std.uniform(gl, "TINT_COLOR"),
            },
            sdf_uniforms: SdfUniforms {
                common: common(gl, &sdf),
                text_color: sdf.uniform(gl, "TEXT_COLOR"),
                outline_color: sdf.uniform(gl, "OUTLINE_COLOR"),
                distance_edges: sdf.uniform(gl, "DISTANCE_EDGES"),
                tex_id: sdf.uniform(gl, "TEX_ID"),
                tex_chan_mask: sdf.uniform(gl, "TEX_CHAN_MASK"),
            },
            std,
            sdf,
        })
    }
}
