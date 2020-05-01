#version 100

uniform vec2 SCREEN_SIZE;

attribute vec2 in_pos;
attribute vec2 in_uv;

#ifdef GL_FRAGMENT_PRECISION_HIGH
varying highp vec2 pass_uv;
#else
varying mediump vec2 pass_uv;
#endif

void main() {
    gl_Position.x = in_pos.x / SCREEN_SIZE.x * 2.0 - 1.0;
    gl_Position.y = in_pos.y / SCREEN_SIZE.y * 2.0 - 1.0;
    pass_uv.x = in_uv.x;
    pass_uv.y = in_uv.y;
}
