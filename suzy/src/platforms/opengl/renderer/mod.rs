/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

mod batch;
mod coverage;
mod vertex;

use std::{convert::TryFrom, ffi::c_void};

use super::{
    opengl_bindings::{
        types::{GLsizei, GLsizeiptr},
        ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER, FALSE, FLOAT, STREAM_DRAW,
        TEXTURE0, TEXTURE_2D, TRIANGLES, TRUE, UNSIGNED_BYTE, UNSIGNED_SHORT,
    },
    shader::ShaderProgram,
};

pub use batch::{Batch, BatchPool, BatchRef};
pub use coverage::{BoundingBox, CoveredArea};
pub use vertex::{
    UvRect, UvRectValues, UvType, Vertex, VertexConfig, VertexVec,
};

pub(super) fn render(ctx: &mut super::OpenGlContext, batches: BatchPool) {
    let want_buffers =
        u16::try_from(2 * batches.batches.len()).unwrap_or(u16::MAX);
    let existing_buffers = ctx.buffers.len();
    if usize::from(want_buffers) > existing_buffers {
        let new_buffers = usize::from(want_buffers) - existing_buffers;
        ctx.buffers.append(&mut vec![0; new_buffers]);
        unsafe {
            ctx.bindings.GenBuffers(
                want_buffers.into(),
                ctx.buffers[existing_buffers..].as_mut_ptr(),
            );
        }
    }

    ctx.shaders.shader.make_current(&ctx.bindings, None);
    ShaderProgram::set_mat4(
        &ctx.bindings,
        ctx.shaders.uniforms.transform,
        batches.matrix.as_ref(),
    );

    let mut buffer_index: u16 = 0;
    for batch in batches.batches {
        let Some((tex_id, tex_size)) = ctx.texture_cache.lookup(&batch.texture) else { continue };
        unsafe {
            ctx.bindings.ActiveTexture(TEXTURE0);
            ctx.bindings.BindTexture(TEXTURE_2D, tex_id);
            ShaderProgram::set_opaque(
                &ctx.bindings,
                ctx.shaders.uniforms.tex_id,
                0,
            );
            ShaderProgram::set_vec2(
                &ctx.bindings,
                ctx.shaders.uniforms.tex_size,
                tex_size.texture_width.into(),
                tex_size.texture_height.into(),
            );
            ShaderProgram::set_float(
                &ctx.bindings,
                ctx.shaders.uniforms.tex_sdf,
                if tex_size.is_sdf { 1.0 } else { 0.0 },
            );
            ctx.bindings.BindBuffer(
                ARRAY_BUFFER,
                ctx.buffers[usize::from(buffer_index)],
            );
            ctx.bindings.BindBuffer(
                ELEMENT_ARRAY_BUFFER,
                ctx.buffers[usize::from(buffer_index) + 1],
            );
            buffer_index = buffer_index.wrapping_add(2);
            let vertex_data_size = batch.vertices.data_size();
            let index_data_size =
                batch.indices.len() * std::mem::size_of::<u16>();
            ctx.bindings.BufferData(
                ARRAY_BUFFER,
                vertex_data_size,
                batch.vertices.data_ptr(),
                STREAM_DRAW,
            );
            ctx.bindings.BufferData(
                ELEMENT_ARRAY_BUFFER,
                index_data_size as GLsizeiptr,
                batch.indices.as_ptr().cast(),
                STREAM_DRAW,
            );
            let offset_info =
                vertex::OffsetInfo::for_vertex_vec(&batch.vertices);
            ctx.bindings.VertexAttribPointer(
                0,
                2,
                FLOAT,
                FALSE,
                offset_info.stride as _,
                offset_info.xy as *const c_void,
            );
            ctx.bindings.VertexAttribPointer(
                1,
                2,
                offset_info.uv_type,
                FALSE,
                offset_info.stride as _,
                offset_info.uv as *const c_void,
            );
            ctx.bindings.VertexAttribPointer(
                2,
                4,
                UNSIGNED_BYTE,
                TRUE,
                offset_info.stride as _,
                offset_info.color as *const c_void,
            );
            ctx.bindings.VertexAttribPointer(
                3,
                4,
                UNSIGNED_BYTE,
                TRUE,
                offset_info.stride as _,
                offset_info.config as *const c_void,
            );
            ctx.bindings.VertexAttribPointer(
                4,
                1,
                FLOAT,
                TRUE,
                offset_info.stride as _,
                offset_info.smoothing as *const c_void,
            );
            ctx.bindings.DrawElements(
                TRIANGLES,
                batch.indices.len() as GLsizei,
                UNSIGNED_SHORT,
                std::ptr::null(),
            );
        }
    }
}
