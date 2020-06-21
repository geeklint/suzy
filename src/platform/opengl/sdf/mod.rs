
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
            outline_color: crate::math::Color::create_rgba8(0xff, 0xff, 0xff, 0),
            distance_edges: (0.49, 0.51, 0.0, 0.0),
            texture: source.texture.clone(),
            tex_chan_mask: (0.0, 0.0, 0.0, 0.0),
        }
    }
}
