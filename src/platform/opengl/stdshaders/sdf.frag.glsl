#version 100

uniform lowp vec4 TEXT_COLOR;

uniform sampler2D TEX_ID;

#ifdef GL_FRAGMENT_PRECISION_HIGH
varying highp vec2 pass_uv;
#else
varying mediump vec2 pass_uv;
#endif

void main() {
    if (texture2D(TEX_ID, pass_uv).a > 0.5) {
        gl_FragColor = TEXT_COLOR;
    } else {
        discard;
    }
}
