#version 330 core

uniform float SCREEN_WIDTH;
uniform float SCREEN_HEIGHT;

layout(location = 0) in vec2 in_pos;
layout(location = 1) in vec2 in_uv;

out VertexData {
    vec2 uv;
} o;

void main() {
    gl_Position.x = in_pos.x / SCREEN_WIDTH * 2.0 - 1.0;
    gl_Position.y = in_pos.y / SCREEN_HEIGHT * 2.0 - 1.0;
    o.uv = in_uv;
}
