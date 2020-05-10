
mod drawparams;
pub use drawparams::DrawParams;

use crate::platform;
use crate::platform::opengl;

pub struct SdfRenderPlatform;

impl platform::RenderPlatform for SdfRenderPlatform {
    type Global = opengl::Gl;
    type DrawParams = DrawParams;
}

impl platform::SubRenderPlatform<opengl::OpenGlRenderPlatform>
for SdfRenderPlatform
{
    fn inherit_params(source: &opengl::drawparams::DrawParams) -> DrawParams {
        DrawParams {
            shaders: source.shaders.clone(),
            transform: source.transform,
            tint_color: source.tint_color,
            text_color: crate::math::consts::WHITE,
            texture: source.texture.clone(),
        }
    }
}
