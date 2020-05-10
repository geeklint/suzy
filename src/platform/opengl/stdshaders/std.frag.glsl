#version 100

uniform sampler2D TEX_ID;
uniform lowp vec4 TINT_COLOR;

#ifdef GL_FRAGMENT_PRECISION_HIGH
varying highp vec2 pass_uv;
#else
varying mediump vec2 pass_uv;
#endif

void main() {
    gl_FragColor = TINT_COLOR * texture2D(TEX_ID, pass_uv);
}
