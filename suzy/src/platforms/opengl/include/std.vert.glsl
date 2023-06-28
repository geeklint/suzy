/* SPDX-License-Identifier: (MIT OR Apache-2.0 OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#version 100

uniform mat4 TRANSFORM;

attribute highp vec2 in_xy;
attribute mediump vec2 in_uv;
attribute lowp vec4 in_color;
attribute mediump vec4 in_config;
attribute mediump float in_smoothing;

varying lowp vec4 pass_color;
varying mediump vec2 pass_config;
varying mediump vec2 pass_uv;
varying lowp vec2 pass_distance;
varying mediump float pass_smoothing;

void main() {
    lowp float dy = float(in_config.z > 0.5);
    lowp float dx = float(max(in_config.z - 0.5 * dy, 0.0) > 0.25);
    mediump vec3 srgb_high = (in_color.rgb + 0.055) / 1.055;
    srgb_high = pow(srgb_high, vec3(2.4, 2.4, 2.4));
    mediump vec3 srgb_low = in_color.rgb / 12.92;
    lowp vec3 srgb_cond = vec3(
        float(in_color.r > 0.04045),
        float(in_color.g > 0.04045),
        float(in_color.b > 0.04045)
    );

    gl_Position = TRANSFORM * vec4(in_xy, 0, 1);
    pass_color = vec4(
        (srgb_high * srgb_cond) + (srgb_low * (1.0 - srgb_cond)),
        in_color.a
    );
    highp float alpha_base = in_config.x;
    alpha_base *= 0.99609375; // (255/256)
    pass_config = vec2(
        alpha_base,
        in_config.y
    );
    pass_uv = in_uv;
    pass_distance = vec2(dx, dy);
    pass_smoothing = in_smoothing;
}
