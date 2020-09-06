#version 100

uniform mat4 TRANSFORM;
uniform vec4 TEX_TRANSFORM;

attribute vec2 in_pos;
attribute vec2 in_uv;

#ifdef GL_FRAGMENT_PRECISION_HIGH
varying highp vec2 pass_uv;
#else
varying mediump vec2 pass_uv;
#endif

void main() {
    gl_Position = TRANSFORM * vec4(in_pos, 0, 1);
    pass_uv.x = in_uv.x / TEX_TRANSFORM.z + TEX_TRANSFORM.x;
    pass_uv.y = in_uv.y / TEX_TRANSFORM.w + TEX_TRANSFORM.y;
}
