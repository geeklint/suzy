#version 100

uniform sampler2D TEX_ID;
uniform lowp vec4 TINT_COLOR;

uniform sampler2D MASK_ID;
uniform mediump vec4 MASK_BOUNDS;

#ifdef GL_FRAGMENT_PRECISION_HIGH
varying highp vec2 pass_uv;
#else
varying mediump vec2 pass_uv;
#endif

void main() {
    mediump vec2 mask_uv = gl_FragCoord.xy / MASK_BOUNDS.zw;
    lowp float mask_alpha = texture2D(MASK_ID, mask_uv).a;
    mask_alpha = (mask_alpha - MASK_BOUNDS.x) * MASK_BOUNDS.y;
    lowp vec4 mask_color = vec4(1, 1, 1, mask_alpha);
    gl_FragColor = mask_color * TINT_COLOR * texture2D(TEX_ID, pass_uv);
}
