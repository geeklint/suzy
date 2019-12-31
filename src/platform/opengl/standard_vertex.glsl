#version 330 core

uniform VUniforms {
    float screen_width;
    float screen_height;
} u;

layout(location = 0) in vec2 in_pos;
layout(location = 1) in vec2 in_uv;

out VertexData {
    vec2 pos;
    vec2 uv;
} o;

void main() {
    o.pos.x = in_pos.x / u.screen_width * 2.0 - 1.0;
    o.pos.y = in_pos.y / u.screen_height * 2.0 - 1.0;
    o.uv = in_uv;
}
