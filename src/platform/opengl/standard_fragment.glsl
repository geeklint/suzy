#version 330 core

uniform vec4 TINT_COLOR;

uniform sampler2D TEX_ID;

in VertexData {
    vec2 uv;
} i;

out vec4 out_color;

void main() {
    out_color = TINT_COLOR * texture(TEX_ID, i.uv);
}
