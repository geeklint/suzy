/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use super::context::bindings::TEXTURE_2D;
use super::context::{OpenGlBindings, OpenGlContext};

mod populate;

pub use populate::*;

pub(super) type TextureCache = HashMap<TextureCacheKey, Rc<SharedTexture>>;

gl_object! { pub(super) TextureData, GenTextures, DeleteTextures, 1 }

/// A type indicating the size of a texture.
///
/// When loaded, textures may be rounded-up to the nearest power of two.
/// This distingueshes between "image size" (the size of the meaninful content)
/// and "texture size", which may be larger due to rounding.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextureSize {
    /// The width of the meaninful image in this texture.
    pub image_width: f32,

    /// The height of the meaninful image in this texture.
    pub image_height: f32,

    /// The actual width of this texture on the GPU.
    pub texture_width: f32,

    /// The actual height of this texture on the GPU.
    pub texture_height: f32,
}

impl TextureSize {
    fn get_crop_transform(
        self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> (f32, f32, f32, f32) {
        let width = if width.is_nan() {
            self.image_width - x
        } else {
            width
        };
        let height = if height.is_nan() {
            self.image_height - y
        } else {
            height
        };
        let offset_x = x / self.texture_width;
        let offset_y = y / self.texture_height;
        let scale_x = self.texture_width / width;
        let scale_y = self.texture_height / height;
        (offset_x, offset_y, scale_x, scale_y)
    }
}

#[derive(Debug)]
pub(super) struct SizedTexture {
    obj: TextureData,
    size: TextureSize,
}

impl SizedTexture {
    fn size(&self) -> Option<&TextureSize> {
        if let Some(true) = self.obj.ready {
            Some(&self.size)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(super) struct SharedTexture {
    obj: RefCell<SizedTexture>,
    populator: Box<dyn PopulateTexture>,
}

impl SharedTexture {
    fn new(populator: Box<dyn PopulateTexture>) -> Rc<Self> {
        let st = SizedTexture {
            obj: TextureData::new(),
            size: TextureSize::default(),
        };
        Rc::new(Self {
            obj: RefCell::new(st),
            populator,
        })
    }

    fn size(&self) -> Option<TextureSize> {
        self.obj.borrow().size().copied()
    }

    fn bind(&self, gl: &Rc<OpenGlBindings>) -> bool {
        let mut borrow = self.obj.borrow_mut();
        let SizedTexture { obj, size } = &mut *borrow;
        let ready = obj.check_ready(gl);
        unsafe {
            gl.BindTexture(TEXTURE_2D, obj.ids[0]);
        }
        if !ready.unwrap_or(false) {
            if let Ok(pop_size) = self.populator.populate(gl) {
                *size = pop_size;
                obj.mark_ready();
            } else {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum KeyVariants {
    Path(Cow<'static, Path>),
    Buffer(*const u8),
    Default,
    Error,
}

impl Default for KeyVariants {
    fn default() -> Self {
        Self::Default
    }
}

/// A key indicating the source of a texture, used to lookup duplicate
/// textures in the texture cache.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextureCacheKey {
    inner: KeyVariants,
}

impl TextureCacheKey {
    fn buffer(buf: &'static [u8]) -> Self {
        Self {
            inner: KeyVariants::Buffer(buf.as_ptr()),
        }
    }

    fn error() -> Self {
        Self {
            inner: KeyVariants::Error,
        }
    }
}

impl From<PathBuf> for TextureCacheKey {
    fn from(path: PathBuf) -> Self {
        Self {
            inner: KeyVariants::Path(path.into()),
        }
    }
}

impl From<&'static Path> for TextureCacheKey {
    fn from(path: &'static Path) -> Self {
        Self {
            inner: KeyVariants::Path(path.into()),
        }
    }
}

impl From<String> for TextureCacheKey {
    fn from(string: String) -> Self {
        let path: PathBuf = string.into();
        path.into()
    }
}

impl From<&'static str> for TextureCacheKey {
    fn from(string: &'static str) -> Self {
        let path: &Path = string.as_ref();
        path.into()
    }
}

#[derive(Clone, Debug)]
enum TextureInstance {
    Existing(Rc<SharedTexture>),
    ToCache(TextureCacheKey, Box<dyn PopulateTexture>),
}

impl TextureInstance {
    fn error() -> Self {
        Self::ToCache(
            TextureCacheKey::error(),
            Box::new(ErrorTexturePopulator),
        )
    }

    fn size(&self) -> Option<TextureSize> {
        match self {
            Self::Existing(existing) => existing.size(),
            Self::ToCache(_, _) => None,
        }
    }

    fn image_size(&self) -> Option<(f32, f32)> {
        match self {
            Self::Existing(existing) => {
                existing.size().map(|ts| (ts.image_width, ts.image_height))
            }
            Self::ToCache(_, pop) => pop.get_known_size(),
        }
    }

    fn bind(&mut self, ctx: &mut OpenGlContext) {
        let success = if let Self::Existing(ref existing) = self {
            existing.bind(&ctx.bindings)
        } else {
            match std::mem::replace(self, Self::error()) {
                // TODO: figure out how to do this without unreachable!() ?
                Self::Existing(_) => unreachable!(),
                Self::ToCache(key, populator) => {
                    let entry = ctx.texture_cache.entry(key);
                    let cached = entry
                        .or_insert_with(move || SharedTexture::new(populator));
                    let clone = Rc::clone(cached);
                    let success = clone.bind(&ctx.bindings);
                    *self = Self::Existing(clone);
                    success
                }
            }
        };
        if !success {
            let entry = ctx.texture_cache.entry(TextureCacheKey::error());
            let cached = entry.or_insert_with(move || {
                SharedTexture::new(Box::new(ErrorTexturePopulator))
            });
            let clone = Rc::clone(cached);
            assert!(clone.bind(&ctx.bindings), "Failed to bind error texture",);
        }
    }
}

impl Default for TextureInstance {
    fn default() -> Self {
        Self::ToCache(
            TextureCacheKey::default(),
            Box::new(DefaultTexturePopulator),
        )
    }
}

impl PartialEq<Self> for TextureInstance {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Existing(ptr), Self::Existing(other_ptr)) => {
                Rc::ptr_eq(ptr, other_ptr)
            }
            (Self::ToCache(key, _), Self::ToCache(other_key, _)) => {
                key == other_key
            }
            _ => false,
        }
    }
}

/// A Texture provides a reference to image data on the GPU.
#[derive(Clone, Debug, PartialEq)]
pub struct Texture {
    ins: TextureInstance,
    offset: (f32, f32),
    size: (f32, f32),
}

impl Texture {
    /// Create a new texture with a custom populator.
    pub fn new<T, U>(populator: T) -> Self
    where
        T: Into<Box<U>>,
        U: 'static + PopulateTexture,
    {
        let populator = populator.into();
        let size = populator.get_known_size().unwrap_or((f32::NAN, f32::NAN));
        Self {
            ins: TextureInstance::Existing(SharedTexture::new(populator)),
            offset: (0.0, 0.0),
            size,
        }
    }

    /// Create a new texture which will be entered into the texture cache.
    ///
    /// Multiple textures created with this function and equal keys will share
    /// the texture resource.
    pub fn new_cached<T, U>(key: TextureCacheKey, populator: T) -> Self
    where
        T: Into<Box<U>>,
        U: 'static + PopulateTexture,
    {
        let populator = populator.into();
        let size = populator.get_known_size().unwrap_or((f32::NAN, f32::NAN));
        Self {
            ins: TextureInstance::ToCache(key, populator),
            offset: (0.0, 0.0),
            size,
        }
    }

    /// Get the size of the texture.  
    ///
    /// May return None if the texture is not fully initialized, and we
    /// cannot determine the size.
    pub fn size(&self) -> Option<(f32, f32)> {
        if !self.size.0.is_nan() {
            Some(self.size)
        } else {
            self.ins.image_size()
        }
    }

    /// Create a texture from some single-channel pixel data
    pub fn from_alpha<T>(
        width: u16,
        height: u16,
        alignment: u16,
        pixels: T,
    ) -> Self
    where
        T: Into<Cow<'static, [u8]>>,
    {
        let pixels = pixels.into();
        assert_eq!(
            pixels.len(),
            PopulateTextureUtil::data_len(width, height, alignment, 1),
            "Invalid pixel data for given width/height/alignment",
        );
        Self::new(AlphaTexturePopulator {
            width,
            height,
            alignment,
            pixels,
        })
    }

    /// Create a texture from some RGB pixel data
    pub fn from_rgb<T>(
        width: u16,
        height: u16,
        alignment: u16,
        pixels: T,
    ) -> Self
    where
        T: Into<Cow<'static, [u8]>>,
    {
        let pixels = pixels.into();
        assert_eq!(
            pixels.len(),
            PopulateTextureUtil::data_len(width, height, alignment, 3),
            "Invalid pixel data for given width/height/alignment",
        );
        Self::new(RgbTexturePopulator {
            width,
            height,
            alignment,
            pixels,
        })
    }

    /// Create a texture from some RGBA pixel data
    pub fn from_rgba<T>(
        width: u16,
        height: u16,
        alignment: u16,
        pixels: T,
    ) -> Self
    where
        T: Into<Cow<'static, [u8]>>,
    {
        let pixels = pixels.into();
        assert_eq!(
            pixels.len(),
            PopulateTextureUtil::data_len(width, height, alignment, 4),
            "Invalid pixel data for given width/height/alignment",
        );
        Self::new(RgbaTexturePopulator {
            width,
            height,
            alignment,
            pixels,
        })
    }

    /// Create a texture from some static RGBA pixel data, sharing a
    /// texture resource if this function is called multiple times with the
    /// same buffer.
    pub fn from_rgba_cached(
        width: u16,
        height: u16,
        alignment: u16,
        pixels: &'static [u8],
    ) -> Self {
        assert_eq!(
            pixels.len(),
            PopulateTextureUtil::data_len(width, height, alignment, 4),
            "Invalid pixel data for given width/height/alignment",
        );
        let key = TextureCacheKey::buffer(pixels);
        Self::new_cached(
            key,
            RgbaTexturePopulator {
                width,
                height,
                alignment,
                pixels: pixels.into(),
            },
        )
    }

    /// Crop this texture.
    ///
    /// This, along with Texture::clone, enables patterns like sprite-sheets
    /// where multiple images are packed into a single texture reference.
    pub fn crop(self, x: f32, y: f32, height: f32, width: f32) -> Self {
        Self {
            ins: self.ins,
            offset: (self.offset.0 + x, self.offset.1 + y),
            size: (height, width),
        }
    }

    pub fn bind(&mut self, ctx: &mut OpenGlContext) -> (f32, f32, f32, f32) {
        self.ins.bind(ctx);
        let size = self.ins.size().expect(
            "Failed to get texture size, even though we just bound it",
        );
        size.get_crop_transform(
            self.offset.0,
            self.offset.1,
            self.size.0,
            self.size.1,
        )
    }

    /// If this texture is ready, call the closure provided, and update
    /// the UV coordinates it returns according to the size and crop settings
    /// of this texture.
    pub fn transform_uvs<'a, F>(&self, make_uvs: F) -> Option<&'a [f32]>
    where
        F: 'a + FnOnce() -> &'a mut [f32],
    {
        self.ins.size().map(|size| {
            let (offset_x, offset_y, scale_x, scale_y) = size
                .get_crop_transform(
                    self.offset.0,
                    self.offset.1,
                    self.size.0,
                    self.size.1,
                );
            let uvs = make_uvs();

            for pair in uvs.chunks_exact_mut(2) {
                pair[0] = (pair[0] / scale_x) + offset_x;
                pair[1] = (pair[1] / scale_y) + offset_y;
            }

            &*uvs
        })
    }
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            ins: TextureInstance::default(),
            offset: (0.0, 0.0),
            size: (2.0, 2.0),
        }
    }
}
