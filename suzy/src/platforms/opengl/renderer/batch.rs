/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::platforms::opengl::{
    texture::TextureCache, Mat4, Texture, TextureId,
};

use super::{vertex::UvRect, BoundingBox, CoveredArea, VertexVec};

pub struct Batch {
    pub(super) texture: TextureId,
    covered_area: CoveredArea,
    pub vertices: VertexVec,
    pub indices: Vec<u16>,
    pub(super) masking: BatchMasking,
}

pub struct BatchRef<'a> {
    pub batch: &'a mut Batch,
    pub uv_rect: UvRect,
}

pub struct BatchPool {
    pub(in crate::platforms::opengl) matrix: Mat4,
    pub(super) batches: Vec<Batch>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchMasking {
    Unmasked,
    NewMask,
    AddToMask,
    Masked,
}

impl BatchPool {
    pub fn new(matrix: Mat4) -> Self {
        Self {
            matrix,
            batches: Vec::new(),
        }
    }

    fn can_use_texture(
        texture_cache: &TextureCache,
        batch_tex: &TextureId,
        tex: &Texture,
    ) -> Option<UvRect> {
        // TODO: special handling for solid_color texture
        if batch_tex == &tex.id() {
            let (_id, texture_size) = texture_cache.lookup(batch_tex)?;
            Some(tex.get_uv_rect(texture_size))
        } else {
            None
        }
    }

    pub(in crate::platforms::opengl) fn find_batch(
        &mut self,
        texture_cache: &TextureCache,
        tex: &Texture,
        masking: BatchMasking,
        num_vertices: u16,
        draw_area: &[BoundingBox],
    ) -> Option<BatchRef<'_>> {
        let mut found: Option<(usize, UvRect)> = None;
        for (i, batch) in self.batches.iter_mut().enumerate().rev() {
            match (batch.masking, masking) {
                (_, BatchMasking::AddToMask) => {
                    panic!("should not create a batch using AddToMask");
                }
                // can render un-masked content before rendering to a mask
                (BatchMasking::AddToMask, BatchMasking::Unmasked)
                | (BatchMasking::NewMask, BatchMasking::Unmasked) => {
                    continue;
                }
                // generally, a new mask should be a new batch
                (BatchMasking::Unmasked, BatchMasking::NewMask)
                | (BatchMasking::Masked, BatchMasking::NewMask)
                // must not render masked content before rendering to a mask
                | (BatchMasking::AddToMask, BatchMasking::Masked)
                | (BatchMasking::NewMask, BatchMasking::Masked) => {
                    break;
                }
                // regular drawing but unmatched masking, can't be merged
                // but can draw past
                (BatchMasking::Unmasked, BatchMasking::Masked)
                | (BatchMasking::Masked, BatchMasking::Unmasked) => {}
                // drawing with matched masking, can be merged
                (BatchMasking::Unmasked, BatchMasking::Unmasked)
                | (BatchMasking::Masked, BatchMasking::Masked)
                | (BatchMasking::NewMask, BatchMasking::NewMask)
                | (BatchMasking::AddToMask, BatchMasking::NewMask) => {
                    if let Some(uv_rect) = Self::can_use_texture(
                        texture_cache,
                        &batch.texture,
                        tex,
                    ) {
                        if batch.vertices.can_add(num_vertices) {
                            batch.vertices.reserve(num_vertices.into());
                            found = Some((i, uv_rect));
                            break;
                        }
                    }
                }
            }
            if draw_area.iter().any(|bb| batch.covered_area.overlaps(bb)) {
                break;
            }
        }
        let found = found.or_else(|| {
            let index = self.batches.len();
            self.batches.push(Batch {
                texture: tex.id(),
                covered_area: CoveredArea::default(),
                masking,
                vertices: VertexVec::default(),
                indices: Vec::new(),
            });
            let (_id, size) = texture_cache.lookup(&tex.id())?;
            let uv_rect = tex.get_uv_rect(size);
            Some((index, uv_rect))
        });
        match found {
            Some((index, uv_rect)) => {
                let batch = &mut self.batches[index];
                for bb in draw_area {
                    batch.covered_area.add_covered(bb);
                }
                Some(BatchRef { batch, uv_rect })
            }
            None => None,
        }
    }

    pub(in crate::platforms::opengl) fn pop_empty_mask(&mut self) {
        let mut remove_after = self.batches.len();
        for (i, batch) in self.batches.iter().enumerate().rev() {
            match batch.masking {
                BatchMasking::Unmasked | BatchMasking::Masked => break,
                BatchMasking::NewMask | BatchMasking::AddToMask => {
                    remove_after = i;
                }
            }
        }
        self.batches.drain(remove_after..);
    }

    pub(in crate::platforms::opengl) fn reduce_mask_clears(&mut self) {
        let mut iter = self.batches.iter_mut();
        while let Some(batch) = iter.next_back() {
            if batch.masking != BatchMasking::NewMask {
                continue;
            }
            if let Some(earlier) = iter.as_slice().last() {
                if earlier.masking == BatchMasking::NewMask {
                    batch.masking = BatchMasking::AddToMask;
                }
            }
        }
    }
}
