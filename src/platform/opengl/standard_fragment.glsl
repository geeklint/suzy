#version 330 core

in VertexData {
    vec2 uv;
} i;

out vec3 out_color;

void main() {
    out_color = vec3(1,0,1);
}
