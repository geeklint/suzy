/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

mod image;

pub use image::SlicedImage;

use super::renderer::BatchPool;

enum DrawPass {
    GatherTextures,
    Main { batch_pool: BatchPool },
}

pub struct DrawContext<'a> {
    context: &'a mut super::context::OpenGlContext,
    pass: DrawPass,
}

impl<'a> crate::graphics::PlatformDrawContext<()> for DrawContext<'a> {
    fn finish(self) -> Option<()> {
        if matches!(self.pass, DrawPass::GatherTextures) {
            self.context.run_texture_populators();
            Some(())
        } else {
            None
        }
    }
}

impl<'a> DrawContext<'a> {
    pub(crate) fn new(
        context: &'a mut super::context::OpenGlContext,
        matrix: super::Mat4,
        first_pass: bool,
    ) -> Self {
        Self {
            context,
            pass: if first_pass {
                DrawPass::GatherTextures
            } else {
                DrawPass::Main {
                    batch_pool: BatchPool::new(matrix),
                }
            },
        }
    }

    pub fn find_batch(
        &mut self,
        tex: &super::Texture,
        num_vertices: u16,
        draw_area: &[super::renderer::BoundingBox],
    ) -> Option<super::renderer::BatchRef<'_>> {
        match &mut self.pass {
            DrawPass::GatherTextures => {
                self.context.texture_cache.register(tex);
                None
            }
            DrawPass::Main { batch_pool } => batch_pool.find_batch(
                &self.context.texture_cache,
                tex,
                num_vertices,
                draw_area,
            ),
        }
    }
}
