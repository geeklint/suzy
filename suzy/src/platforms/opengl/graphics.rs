/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

mod image;
mod mask;

pub use {image::SlicedImage, mask::Mask};

use super::renderer::{BatchMasking, BatchPool};

enum DrawPass {
    GatherTextures,
    Main {
        masking: BatchMasking,
        batch_pool: BatchPool,
    },
}

pub struct DrawContext<'a> {
    context: &'a mut super::context::OpenGlContext,
    pass: DrawPass,
}

impl<'a> crate::graphics::PlatformDrawContext<()> for DrawContext<'a> {
    fn finish(self) -> Option<()> {
        match self.pass {
            DrawPass::GatherTextures => {
                self.context.run_texture_populators();
                Some(())
            }
            DrawPass::Main { batch_pool, .. } => {
                super::renderer::render(self.context, batch_pool);
                None
            }
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
                    masking: BatchMasking::Unmasked,
                    batch_pool: BatchPool::new(matrix),
                }
            },
        }
    }

    pub fn push_mask(&mut self) {
        match &mut self.pass {
            DrawPass::GatherTextures => {}
            DrawPass::Main { masking, .. } => match masking {
                BatchMasking::Unmasked => *masking = BatchMasking::NewMask,
                _ => panic!(
                    "attempting to push a mask while one was already pushed"
                ),
            },
        }
    }

    pub fn start_masking(&mut self) {
        match &mut self.pass {
            DrawPass::GatherTextures => {}
            DrawPass::Main { masking, .. } => match masking {
                BatchMasking::NewMask
                | BatchMasking::AddToMask
                | BatchMasking::Masked => *masking = BatchMasking::Masked,
                BatchMasking::Unmasked => panic!(
                    "attempted to start masking without first pushing a mask",
                ),
            },
        }
    }

    pub fn pop_mask(&mut self) {
        match &mut self.pass {
            DrawPass::GatherTextures => {}
            DrawPass::Main {
                masking,
                batch_pool,
            } => match masking {
                BatchMasking::NewMask
                | BatchMasking::AddToMask
                | BatchMasking::Masked => {
                    *masking = BatchMasking::Unmasked;
                    batch_pool.pop_empty_mask();
                }
                BatchMasking::Unmasked => panic!(
                    "attempted to pop a mask without first pushing a mask",
                ),
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
            DrawPass::Main {
                masking,
                batch_pool,
            } => batch_pool.find_batch(
                &self.context.texture_cache,
                tex,
                *masking,
                num_vertices,
                draw_area,
            ),
        }
    }
}
