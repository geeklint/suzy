/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{borrow::Borrow, collections::HashMap, hash::Hash, rc::Rc};

use super::{
    context::{
        bindings::TEXTURE_2D,
        short_consts::{CLAMP_TO_EDGE, NEAREST, RGBA},
        OpenGlBindings,
    },
    opengl_bindings::{
        types::GLuint, TEXTURE_MAG_FILTER, TEXTURE_MIN_FILTER, TEXTURE_WRAP_S,
        TEXTURE_WRAP_T, UNPACK_ALIGNMENT, UNSIGNED_BYTE,
    },
    renderer::{UvRect, UvRectValues},
};

mod populate;

pub use populate::{PopulateTexture, PopulateTextureUtil};

#[derive(Clone, Debug)]
pub struct Texture {
    populator: Option<Rc<dyn PopulateTexture>>,
    pub(super) crop: Option<UvRect>,
    //fallback: Fallback,
}

impl Texture {
    pub fn new(populator: Rc<dyn PopulateTexture>) -> Self {
        Self {
            populator: Some(populator),
            crop: None,
            //fallback: Fallback::NoRender,
        }
    }

    #[must_use]
    pub const fn solid_color() -> Self {
        Self {
            populator: None,
            crop: Some(UvRect::SolidColor(0, 0)),
            //fallback: Fallback::SolidColor,
        }
    }

    #[must_use]
    pub fn id(&self) -> TextureId {
        TextureId {
            populator: self.populator.clone(),
        }
    }

    /// Crop this texture.
    ///
    /// This, along with [`Texture::clone`], enables patterns like sprite-sheets
    /// where multiple images are packed into a single texture reference.
    #[must_use]
    pub fn crop(self, left: f32, right: f32, bottom: f32, top: f32) -> Self {
        let (origin_x, origin_y) = match self.crop {
            Some(rect) => match rect {
                UvRect::F32(UvRectValues { left, bottom, .. }) => {
                    (left, bottom)
                }
                UvRect::U16(UvRectValues { left, bottom, .. }) => {
                    (left.into(), bottom.into())
                }
                UvRect::SolidColor(..) => return self,
            },
            None => (0.0, 0.0),
        };
        Self {
            crop: Some(UvRect::F32(UvRectValues {
                left: left + origin_x,
                right: right + origin_x,
                bottom: bottom + origin_y,
                top: top + origin_y,
            })),
            ..self
        }
    }

    pub(super) fn get_uv_rect(
        &self,
        size: &TextureSize,
    ) -> super::renderer::UvRect {
        match self.crop {
            None => size.default_rect,
            Some(uvrect) => uvrect,
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

/*
#[derive(Clone, Debug, Default)]
enum Fallback {
    #[default]
    NoRender,
    SolidColor,
    Texture(Box<Texture>),
}
*/

#[derive(Clone, Copy, Debug)]
enum TextureState {
    Loading,
    Failed,
    Ready { id: GLuint, size: TextureSize },
}

#[derive(Clone, Copy, Debug)]
pub struct TextureSize {
    /// In the absence of Texture::crop, the UvRect to use. This usually
    /// corrosponds to (0, 0)..(w, h) where w and h are the size of the original
    /// image (not rounded up even if the texture size was rounded up to a power
    /// of 2).
    pub default_rect: UvRect,

    /// UVs passed from vertices are divided by this value. Usually this is the
    /// size of the texture data on the GPU, but other uses may be desirable -
    /// 2x the size of the texture data on the GPU allows addressing the center
    /// of texels with integers rather than just the texel boundaries. If the
    /// kind of UV mapping normally found in games is desired, [1, 1] can be
    /// used.
    pub uv_scale: [u16; 2],

    /// If this image represents a signed distance field.
    pub is_sdf: bool,
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
    solid_color: Option<TextureState>,
}

impl TextureCache {
    pub fn lookup(&self, id: &TextureId) -> Option<(GLuint, &TextureSize)> {
        let slot = match id.populator.as_ref() {
            Some(pop) => self.set.get(pop.texture_key()),
            None => self.solid_color.as_ref(),
        };
        slot.and_then(|state| {
            if let TextureState::Ready { id, size } = state {
                Some((*id, size))
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
                        gl.GenTextures(1, &raw mut id_slot);
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
                        eprintln!("failed to load texture: {msg}");
                        new_tex_id = Some(id);
                        *state = TextureState::Failed;
                    }
                }
            }
        }
        self.solid_color.get_or_insert_with(|| {
            let pixels: [u8; 16] = [0xff; 16];
            let id = new_tex_id.take().unwrap_or_else(|| {
                let mut id_slot = 0;
                unsafe {
                    gl.GenTextures(1, &raw mut id_slot);
                }
                id_slot
            });
            unsafe {
                gl.BindTexture(TEXTURE_2D, id);
                gl.PixelStorei(UNPACK_ALIGNMENT, 1);
                gl.TexImage2D(
                    TEXTURE_2D,
                    0,
                    RGBA.into(),
                    2,
                    2,
                    0,
                    RGBA.into(),
                    UNSIGNED_BYTE,
                    pixels.as_ptr().cast(),
                );
                gl.TexParameteri(
                    TEXTURE_2D,
                    TEXTURE_MIN_FILTER,
                    NEAREST.into(),
                );
                gl.TexParameteri(
                    TEXTURE_2D,
                    TEXTURE_MAG_FILTER,
                    NEAREST.into(),
                );
                gl.TexParameteri(
                    TEXTURE_2D,
                    TEXTURE_WRAP_S,
                    CLAMP_TO_EDGE.into(),
                );
                gl.TexParameteri(
                    TEXTURE_2D,
                    TEXTURE_WRAP_T,
                    CLAMP_TO_EDGE.into(),
                );
            }
            TextureState::Ready {
                id,
                size: TextureSize {
                    default_rect: UvRect::SolidColor(0, 0),
                    uv_scale: [2, 2],
                    is_sdf: false,
                },
            }
        });
        if let Some(id) = new_tex_id {
            unsafe {
                gl.DeleteTextures(1, &raw const id);
            }
        }
    }
}
