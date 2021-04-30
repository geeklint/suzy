/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#version 100

uniform lowp vec4 TEXT_COLOR;
uniform lowp vec4 OUTLINE_COLOR;
uniform mediump vec4 DISTANCE_EDGES;

uniform sampler2D TEX_ID;
uniform lowp vec4 TEX_CHAN_MASK;

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
    lowp float value = 1.0 - length(TEX_CHAN_MASK * texture2D(TEX_ID, pass_uv));
    if (value < DISTANCE_EDGES.x) {
        gl_FragColor = mask_color * TEXT_COLOR;
    } else if (value < DISTANCE_EDGES.y) {
        lowp float t = smoothstep(DISTANCE_EDGES.x, DISTANCE_EDGES.y, value);
        gl_FragColor = mask_color * mix(TEXT_COLOR, OUTLINE_COLOR, t);
    } else if (value < DISTANCE_EDGES.z) {
        gl_FragColor = mask_color * OUTLINE_COLOR;
    } else if (value < DISTANCE_EDGES.w) {
        lowp float a = 1.0 - smoothstep(DISTANCE_EDGES.z, DISTANCE_EDGES.w, value);
        gl_FragColor = mask_color * vec4(OUTLINE_COLOR.xyz, a);
    } else {
        discard;
    }
}
