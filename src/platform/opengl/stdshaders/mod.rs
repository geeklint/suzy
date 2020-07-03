use std::rc::Rc;

use super::OpenGlBindings;
use super::shader::{
    ProgramCompileError,
    Shader,
    UniformLoc,
};

const STD_VERTEX_SOURCE: &'static [u8] = include_bytes!(
    "std.vert.glsl");
const STD_FRAGMENT_SOURCE: &'static [u8] = include_bytes!(
    "std.frag.glsl");
const SDF_FRAGMENT_SOURCE: &'static [u8] = include_bytes!(
    "sdf.frag.glsl");

#[derive(Clone, Copy, Debug)]
pub(super) struct VertUniforms {
    pub(super) transform: UniformLoc,
    pub(super) tex_transform: UniformLoc,
}

fn common(gl: &OpenGlBindings, shader: &Shader) -> VertUniforms {
    VertUniforms {
        transform: shader.uniform(gl, "TRANSFORM"),
        tex_transform: shader.uniform(gl, "TEX_TRANSFORM"),
    }
}
    
#[derive(Clone, Copy, Debug)]
pub(super) struct StdUniforms {
    pub(super) common: VertUniforms,
    pub(super) tex_id: UniformLoc,
    pub(super) tint_color: UniformLoc,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SdfUniforms {
    pub(super) common: VertUniforms,
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
