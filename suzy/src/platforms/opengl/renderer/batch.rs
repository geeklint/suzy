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
}

pub struct BatchRef<'a> {
    pub batch: &'a mut Batch,
    pub uv_rect: UvRect,
}

pub struct BatchPool {
    pub(super) matrix: Mat4,
    pub(super) batches: Vec<Batch>,
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
        num_vertices: u16,
        draw_area: &[BoundingBox],
    ) -> Option<BatchRef<'_>> {
        let mut found: Option<(usize, UvRect)> = None;
        for (i, batch) in self.batches.iter_mut().enumerate().rev() {
            if let Some(uv_rect) =
                Self::can_use_texture(texture_cache, &batch.texture, tex)
            {
                if batch.vertices.can_add(num_vertices) {
                    batch.vertices.reserve(num_vertices.into());
                    found = Some((i, uv_rect));
                    break;
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
                vertices: VertexVec::default(),
                indices: Vec::new(),
            });
            let uv_rect = texture_cache
                .lookup(&tex.id())
                .map(|(_id, size)| tex.get_uv_rect(size))?;
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
}
