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

impl From<&Shader> for VertUniforms {
    fn from(shader: &Shader) -> Self {
        Self {
            transform: shader.uniform("TRANSFORM"),
            tex_transform: shader.uniform("TEX_TRANSFORM"),
        }
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
    pub(super) tex_id: UniformLoc,
    pub(super) text_color: UniformLoc,
}

#[derive(Clone)]
pub struct Shaders {
    pub(super) std: Shader,
    pub(super) sdf: Shader,
    pub(super) std_uniforms: StdUniforms,
    pub(super) sdf_uniforms: SdfUniforms,
}

impl Shaders {
    pub fn new() -> Result<Self, ProgramCompileError> {
        let std = Shader::create(STD_VERTEX_SOURCE, STD_FRAGMENT_SOURCE)?;
        let sdf = Shader::create(STD_VERTEX_SOURCE, SDF_FRAGMENT_SOURCE)?;
        Ok(Self {
            std_uniforms: StdUniforms {
                common: (&std).into(),
                tex_id: std.uniform("TEX_ID"),
                tint_color: std.uniform("TINT_COLOR"),
            },
            sdf_uniforms: SdfUniforms {
                common: (&sdf).into(),
                tex_id: sdf.uniform("TEX_ID"),
                text_color: sdf.uniform("TEXT_COLOR"),
            },
            std,
            sdf,
        })
    }
}
