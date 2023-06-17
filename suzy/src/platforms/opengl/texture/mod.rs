/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{borrow::Borrow, collections::HashMap, hash::Hash, rc::Rc};

use super::{
    context::{
        bindings::TEXTURE_2D,
        {OpenGlBindings, OpenGlContext},
    },
    opengl_bindings::types::GLuint,
};

mod populate;

pub use populate::*;

#[derive(Clone, Debug)]
pub struct Texture {
    populator: Option<Rc<dyn PopulateTexture>>,
    pub(super) crop: Crop,
    fallback: Fallback,
}

impl Texture {
    pub fn new(populator: Rc<dyn PopulateTexture>) -> Self {
        Self {
            populator: Some(populator),
            crop: Crop::None,
            fallback: Fallback::NoRender,
        }
    }

    pub fn solid_color() -> Self {
        Self {
            populator: None,
            crop: Crop::None,
            fallback: Fallback::SolidColor,
        }
    }

    pub fn id(&self) -> TextureId {
        TextureId {
            populator: self.populator.clone(),
        }
    }

    /// Crop this texture.
    ///
    /// This, along with Texture::clone, enables patterns like sprite-sheets
    /// where multiple images are packed into a single texture reference.
    pub fn crop(self, x: f32, y: f32, height: f32, width: f32) -> Self {
        Self {
            crop: Crop::F32(CropValues {
                offset_x: x,
                offset_y: y,
                width,
                height,
            }),
            ..self
        }
    }
}

impl Default for Texture {
    fn default() -> Self {
        Self::solid_color()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextureId {
    populator: Option<Rc<dyn PopulateTexture>>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CropValues<T> {
    offset_x: T,
    offset_y: T,
    width: T,
    height: T,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) enum Crop {
    #[default]
    None,
    F32(CropValues<f32>),
    U16(CropValues<u16>),
}

impl Crop {
    pub fn get_uv_rect(&self, size: &TextureSize) -> super::renderer::UvRect {
        todo!()
    }
}

#[derive(Clone, Debug, Default)]
enum Fallback {
    #[default]
    NoRender,
    SolidColor,
    Fallback(Box<Texture>),
}

#[derive(Clone, Copy, Debug)]
enum TextureState {
    Loading,
    Failed,
    Ready { id: GLuint, size: TextureSize },
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextureSize {
    /// The width of the meaninful image in this texture.
    pub image_width: f32,

    /// The height of the meaninful image in this texture.
    pub image_height: f32,

    /// The actual width of this texture on the GPU.
    pub texture_width: u16,

    /// The actual height of this texture on the GPU.
    pub texture_height: u16,
}

#[derive(Debug)]
struct CacheKey {
    populator: Rc<dyn PopulateTexture>,
}

impl Borrow<[u8]> for CacheKey {
    fn borrow(&self) -> &[u8] {
        self.populator.texture_key()
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.populator.texture_key() == other.populator.texture_key()
    }
}

impl Eq for CacheKey {}

impl Hash for CacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.populator.texture_key().hash(state);
    }
}

#[derive(Default)]
pub(super) struct TextureCache {
    set: HashMap<CacheKey, TextureState>,
}

impl TextureCache {
    pub fn lookup(&self, id: &TextureId) -> Option<&TextureSize> {
        self.set
            .get(id.populator.as_ref()?.texture_key())
            .and_then(|state| {
                if let TextureState::Ready { size, .. } = state {
                    Some(size)
                } else {
                    None
                }
            })
    }

    pub fn register(&mut self, tex: &Texture) {
        if let Some(populator) = &tex.populator {
            if !self.set.contains_key(populator.texture_key()) {
                let key = CacheKey {
                    populator: Rc::clone(populator),
                };
                let value = TextureState::Loading;
                self.set.insert(key, value);
            }
        }
    }

    pub fn run_populators(&mut self, gl: &OpenGlBindings) {
        let mut new_tex_id = None;
        for (key, state) in &mut self.set {
            if matches!(state, TextureState::Loading) {
                let id = new_tex_id.take().unwrap_or_else(|| {
                    let mut id_slot = 0;
                    unsafe {
                        gl.GenTextures(1, &mut id_slot);
                    }
                    id_slot
                });
                unsafe {
                    gl.BindTexture(TEXTURE_2D, id);
                }
                match key.populator.populate(gl, TEXTURE_2D) {
                    Ok(size) => {
                        *state = TextureState::Ready { id, size };
                    }
                    Err(msg) => {
                        eprintln!("failed to load texture: {}", msg);
                        new_tex_id = Some(id);
                        *state = TextureState::Failed;
                    }
                }
            }
        }
        if let Some(id) = new_tex_id {
            unsafe {
                gl.DeleteTextures(1, &id);
            }
        }
    }
}
