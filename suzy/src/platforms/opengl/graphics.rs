/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

mod circle;
mod image;
mod line;
mod mask;
mod transform;

pub use {
    circle::Circle, image::SlicedImage, line::Line, mask::Mask,
    transform::Transform,
};

use super::{
    renderer::{BatchMasking, BatchPool},
    Mat4,
};

enum DrawPass<'a> {
    GatherTextures,
    Main {
        masking: BatchMasking,
        batch_pool: &'a mut BatchPool,
    },
}

pub struct DrawContext<'a> {
    context: &'a mut super::context::OpenGlContext,
    pass: DrawPass<'a>,
}

impl<'a> DrawContext<'a> {
    pub(crate) fn gather_textures(
        context: &'a mut super::context::OpenGlContext,
    ) -> Self {
        Self {
            context,
            pass: DrawPass::GatherTextures,
        }
    }

    pub(crate) fn main_draw_pass(
        context: &'a mut super::context::OpenGlContext,
        batch_pool: &'a mut BatchPool,
    ) -> Self {
        Self {
            context,
            pass: DrawPass::Main {
                masking: BatchMasking::Unmasked,
                batch_pool,
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

    pub fn update_matrix<F>(&mut self, f: F)
    where
        F: FnOnce(Mat4) -> Mat4,
    {
        match &mut self.pass {
            DrawPass::GatherTextures => (),
            DrawPass::Main { batch_pool, .. } => {
                let new_pool = BatchPool::new(f(batch_pool.matrix));
                let old_pool = std::mem::replace(*batch_pool, new_pool);
                super::renderer::render(self.context, old_pool);
            }
        }
    }
}
