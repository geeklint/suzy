/* SPDX-License-Identifier: (MIT OR Apache-2.0 OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#version 100

#if GL_OES_standard_derivatives
#extension GL_OES_standard_derivatives : enable
#endif

#define SDF_OFFSET (pass_config.x)
#define SDF_PEAK (pass_config.y)
#define ONE_THIRD (0.33333333)

uniform sampler2D TEX_ID;
uniform mediump vec2 TEX_SIZE;
uniform lowp float TEX_SDF;
uniform mediump float TEX_COLOR_POW;
uniform sampler2D MASK_ID;
uniform mediump vec2 MASK_SIZE;

varying lowp vec4 pass_color;
varying mediump vec2 pass_config;
varying mediump vec2 pass_uv;
varying lowp vec2 pass_distance;
varying mediump float pass_smoothing;

void main() {
    lowp vec4 tex_color = texture2D(TEX_ID, pass_uv / TEX_SIZE);
    mediump vec3 sdf_value = vec3(max(tex_color.a, 1.0 - TEX_SDF));
    sdf_value *= 1.0 - min(length(pass_distance), 1.0);

#if GL_OES_standard_derivatives
    mediump vec2 subpixel_uv_offset = ONE_THIRD * dFdx(pass_uv);
    mediump vec2 subpixel_dist_offset = ONE_THIRD * dFdx(pass_distance);
    tex_color.ra = texture2D(TEX_ID, (pass_uv - subpixel_uv_offset) / TEX_SIZE).ra;
    sdf_value.r = max(tex_color.a, 1.0 - TEX_SDF);
    sdf_value.r *= 1.0 - min(length(pass_distance - subpixel_dist_offset), 1.0);
    tex_color.ba = texture2D(TEX_ID, (pass_uv + subpixel_uv_offset) / TEX_SIZE).ba;
    sdf_value.b = max(tex_color.a, 1.0 - TEX_SDF);
    sdf_value.b *= 1.0 - min(length(pass_distance + subpixel_dist_offset), 1.0);
#endif

    tex_color = max(tex_color, TEX_SDF);
    tex_color.rgb = pow(tex_color.rgb, vec3(TEX_COLOR_POW));
    tex_color.rgb = tex_color.rgb * pass_color.rgb;
    sdf_value = SDF_PEAK - abs(sdf_value - SDF_PEAK);
    sdf_value = sdf_value + (SDF_OFFSET + SDF_OFFSET) - 1.0;
    sdf_value = sdf_value * pass_smoothing;
    sdf_value = clamp(sdf_value, 0.0, 1.0);
    sdf_value *= pass_color.a;
    sdf_value *= texture2D(MASK_ID, gl_FragCoord.xy / MASK_SIZE).a;
    gl_FragColor = vec4(tex_color.rgb * sdf_value, sdf_value);
}
