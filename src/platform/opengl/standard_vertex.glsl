#version 330 core

uniform vec2 SCREEN_SIZE;
uniform vec2 TEX_OFFSET;
uniform vec2 TEX_SCALE;

layout(location = 0) in vec2 in_pos;
layout(location = 1) in vec2 in_uv;

out VertexData {
    vec2 uv;
} o;

void main() {
    gl_Position.x = in_pos.x / SCREEN_SIZE.x * 2.0 - 1.0;
    gl_Position.y = in_pos.y / SCREEN_SIZE.y * 2.0 - 1.0;
    o.uv.x = in_uv.x / TEX_SCALE.x + TEX_OFFSET.x;
    o.uv.y = in_uv.y / TEX_SCALE.y + TEX_OFFSET.y;
}
