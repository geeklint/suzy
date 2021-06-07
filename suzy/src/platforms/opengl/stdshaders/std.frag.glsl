/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#version 100

uniform sampler2D TEX_ID;
uniform lowp vec4 TINT_COLOR;

uniform sampler2D MASK_ID;
uniform mediump vec4 MASK_BOUNDS;

uniform mediump vec4 SDF_VALUES;
uniform lowp vec4 SDF_CHAN_MASK;

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

    lowp vec4 tex_color = texture2D(TEX_ID, pass_uv);

    lowp float sdf_value = 1.0 - dot(tex_color, SDF_CHAN_MASK);
    lowp float sdf_alpha = 1.0 - smoothstep(SDF_VALUES.x, SDF_VALUES.y, sdf_value);
    lowp vec4 sdf_color = vec4(1, 1, 1, sdf_alpha);

    lowp vec4 main_color = mix(tex_color, sdf_color, SDF_VALUES.z);

    gl_FragColor = mask_color * TINT_COLOR * main_color;
}
