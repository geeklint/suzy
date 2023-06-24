/* SPDX-License-Identifier: (MIT OR Apache-2.0 OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#version 100

uniform sampler2D TEX_ID;
uniform mediump vec2 TEX_SIZE;
uniform lowp float TEX_SDF;

varying mediump vec2 pass_uv;
varying lowp vec4 pass_color;
varying mediump vec4 pass_config;
varying mediump float pass_smoothing;

void main() {
    lowp vec4 tex_color = texture2D(TEX_ID, pass_uv / TEX_SIZE);
    mediump float sdf_value = max(tex_color.a, 1.0 - TEX_SDF);
    tex_color = max(tex_color, TEX_SDF);
    sdf_value = pass_config.y - abs(sdf_value - pass_config.y);
    sdf_value = sdf_value + pass_config.x + pass_config.x - 1.0;
    sdf_value = sdf_value * pass_smoothing;
    lowp float alpha = clamp(sdf_value, 0.0, 1.0);
    alpha *= pass_color.a;
    gl_FragColor = vec4(tex_color.rgb * pass_color.rgb, alpha);
}
