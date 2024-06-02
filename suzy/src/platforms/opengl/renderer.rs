/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

mod batch;
mod coverage;
mod vertex;

use std::convert::TryFrom;

use super::{
    opengl_bindings::{
        types::{GLint, GLsizei, GLsizeiptr, GLuint},
        ARRAY_BUFFER, COLOR_BUFFER_BIT, COLOR_CLEAR_VALUE,
        ELEMENT_ARRAY_BUFFER, FALSE, FLOAT, FRAMEBUFFER, FRAMEBUFFER_BINDING,
        STREAM_DRAW, TEXTURE0, TEXTURE1, TEXTURE_2D, TRIANGLES, TRUE,
        UNSIGNED_BYTE, UNSIGNED_SHORT,
    },
    shader::ShaderProgram,
    Texture,
};

pub use batch::{Batch, BatchRef};
pub use coverage::{BoundingBox, CoveredArea};
pub use vertex::{
    UvRect, UvRectValues, UvType, Vertex, VertexConfig, VertexVec,
};

pub(super) use batch::{BatchMasking, BatchPool};

pub(super) fn render(ctx: &mut super::OpenGlContext, mut batches: BatchPool) {
    batches.reduce_mask_clears();

    let want_buffers =
        u16::try_from(2 * batches.batches.len()).unwrap_or(u16::MAX);
    let existing_buffers =
        u16::try_from(ctx.buffers.len()).unwrap_or(want_buffers);
    if want_buffers > existing_buffers {
        let new_buffers = want_buffers - existing_buffers;
        ctx.buffers.append(&mut vec![0; usize::from(new_buffers)]);
        let tail = &mut ctx.buffers[usize::from(existing_buffers)..];
        unsafe {
            ctx.bindings
                .GenBuffers(new_buffers.into(), tail.as_mut_ptr());
        }
    }

    ctx.shaders.shader.make_current(&ctx.bindings, None);
    ShaderProgram::set_mat4(
        &ctx.bindings,
        ctx.shaders.uniforms.transform,
        batches.matrix.as_ref(),
    );
    ShaderProgram::set_vec2(
        &ctx.bindings,
        ctx.shaders.uniforms.mask_size,
        ctx.mask.width,
        ctx.mask.height,
    );

    let Some((solid_color_tex_id, _)) =
        ctx.texture_cache.lookup(&Texture::solid_color().id())
    else {
        return;
    };
    let mut main_fbo = None;

    let mut main_clear_color = [0f32; 4];
    unsafe {
        ctx.bindings
            .GetFloatv(COLOR_CLEAR_VALUE, main_clear_color.as_mut_ptr());
        ctx.bindings.ClearColor(0.0, 0.0, 0.0, 0.0);
    }

    let mut buffer_index: u16 = 0;
    for batch in batches.batches {
        let Some((tex_id, tex_size)) =
            ctx.texture_cache.lookup(&batch.texture)
        else {
            continue;
        };
        let tex_id_for_mask_uniform = match batch.masking {
            BatchMasking::Masked => ctx.mask.texture,
            _ => solid_color_tex_id,
        };
        unsafe {
            match (main_fbo, batch.masking) {
                (None, BatchMasking::NewMask | BatchMasking::AddToMask) => {
                    let mut current_fbo: GLint = 0;
                    ctx.bindings
                        .GetIntegerv(FRAMEBUFFER_BINDING, &mut current_fbo);
                    ctx.bindings.BindFramebuffer(FRAMEBUFFER, ctx.mask.fbo);
                    main_fbo = Some(current_fbo as GLuint);
                }
                (Some(fbo), BatchMasking::Unmasked | BatchMasking::Masked) => {
                    ctx.bindings.BindFramebuffer(FRAMEBUFFER, fbo);
                    main_fbo = None;
                }
                _ => (),
            }
            if batch.masking == BatchMasking::NewMask {
                ctx.bindings.Clear(COLOR_BUFFER_BIT);
            }
            ctx.bindings.ActiveTexture(TEXTURE1);
            ctx.bindings
                .BindTexture(TEXTURE_2D, tex_id_for_mask_uniform);
            ShaderProgram::set_opaque(
                &ctx.bindings,
                ctx.shaders.uniforms.mask_id,
                1,
            );
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
            ShaderProgram::set_float(
                &ctx.bindings,
                ctx.shaders.uniforms.tex_color_pow,
                tex_size.color_pow,
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
            let index_data_size = GLsizeiptr::try_from(
                batch.indices.len() * std::mem::size_of::<u16>(),
            )
            .expect("size of index buffer should fit in GLsizeiptr");
            ctx.bindings.BufferData(
                ARRAY_BUFFER,
                vertex_data_size,
                batch.vertices.data_ptr(),
                STREAM_DRAW,
            );
            ctx.bindings.BufferData(
                ELEMENT_ARRAY_BUFFER,
                index_data_size,
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
                offset_info.stride,
                offset_info.xy,
            );
            ctx.bindings.VertexAttribPointer(
                1,
                2,
                offset_info.uv_type,
                FALSE,
                offset_info.stride,
                offset_info.uv,
            );
            ctx.bindings.VertexAttribPointer(
                2,
                4,
                UNSIGNED_BYTE,
                TRUE,
                offset_info.stride,
                offset_info.color,
            );
            ctx.bindings.VertexAttribPointer(
                3,
                4,
                UNSIGNED_BYTE,
                TRUE,
                offset_info.stride,
                offset_info.config,
            );
            ctx.bindings.VertexAttribPointer(
                4,
                1,
                FLOAT,
                TRUE,
                offset_info.stride,
                offset_info.smoothing,
            );
            ctx.bindings.DrawElements(
                TRIANGLES,
                GLsizei::try_from(batch.indices.len()).unwrap_or(GLsizei::MAX),
                UNSIGNED_SHORT,
                std::ptr::null(),
            );
        }
    }

    unsafe {
        let [r, g, b, a] = main_clear_color;
        ctx.bindings.ClearColor(r, g, b, a);
    }
}
