#version 330 core

uniform vec4 TEXT_COLOR;

uniform sampler2D TEX_ID;

in VertexData {
    vec2 uv;
} i;

layout(location = 0) out vec4 out_color;

void main() {
    if (texture(TEX_ID, i.uv).x > 0.5) {
        out_color = TEXT_COLOR;
    } else {
        discard;
    }
}
