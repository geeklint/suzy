#version 100

uniform lowp vec4 TEXT_COLOR;
uniform lowp vec4 OUTLINE_COLOR;
uniform mediump vec4 DISTANCE_EDGES;

uniform sampler2D TEX_ID;
uniform lowp vec4 TEX_CHAN_MASK;

#ifdef GL_FRAGMENT_PRECISION_HIGH
varying highp vec2 pass_uv;
#else
varying mediump vec2 pass_uv;
#endif

void main() {
    lowp float value = 1.0 - length(TEX_CHAN_MASK * texture2D(TEX_ID, pass_uv));
    if (value < DISTANCE_EDGES.x) {
        gl_FragColor = TEXT_COLOR;
    } else if (value < DISTANCE_EDGES.y) {
        lowp float t = smoothstep(DISTANCE_EDGES.x, DISTANCE_EDGES.y, value);
        gl_FragColor = mix(TEXT_COLOR, OUTLINE_COLOR, t);
    } else if (value < DISTANCE_EDGES.z) {
        gl_FragColor = OUTLINE_COLOR;
    } else if (value < DISTANCE_EDGES.w) {
        lowp float a = 1.0 - smoothstep(DISTANCE_EDGES.z, DISTANCE_EDGES.w, value);
        gl_FragColor = vec4(OUTLINE_COLOR.xyz, a);
    } else {
        discard;
    }
}
