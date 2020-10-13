/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#version 100

uniform mat4 TRANSFORM;

attribute vec2 in_pos;
attribute vec2 in_uv;

#ifdef GL_FRAGMENT_PRECISION_HIGH
varying highp vec2 pass_uv;
#else
varying mediump vec2 pass_uv;
#endif

void main() {
    gl_Position = TRANSFORM * vec4(in_pos, 0, 1);
    pass_uv = in_uv;
}
