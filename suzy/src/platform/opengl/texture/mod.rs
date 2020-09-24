/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::context::{OpenGlContext, OpenGlBindings};
use super::context::bindings::{
    TEXTURE_2D,
};

mod populate;

pub use populate::*;

pub(super) type TextureCache = HashMap<TextureCacheKey, Rc<SharedTexture>>;

gl_object! { pub(super) TextureData, GenTextures, DeleteTextures, 1 }

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextureSize {
    pub image_width: f32,
    pub image_height: f32,
    pub texture_width: f32,
    pub texture_height: f32,
}

impl TextureSize {
    fn get_crop_transform(self, x: f32, y: f32, width: f32, height: f32)
        -> (f32, f32, f32, f32)
    {
        let width = if width.is_nan() { self.image_width - x } else { width };
        let height = if height.is_nan() { self.image_height - y } else { height };
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
            populator
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
    Default,
    Error,
}

impl Default for KeyVariants {
    fn default() -> Self { Self::Default }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextureCacheKey {
    inner: KeyVariants,
}

impl TextureCacheKey {
    fn error() -> Self {
        Self { inner: KeyVariants::Error }
    }
}

impl From<PathBuf> for TextureCacheKey {
    fn from(path: PathBuf) -> Self {
        Self { inner: KeyVariants::Path(path.into()) }
    }
}

impl From<&'static Path> for TextureCacheKey {
    fn from(path: &'static Path) -> Self {
        Self { inner: KeyVariants::Path(path.into()) }
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
            Self::Existing(existing) => existing.size()
                .map(|ts| (ts.image_width, ts.image_height)
            ),
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
                    let cached = entry.or_insert_with(move || {
                        SharedTexture::new(populator)
                    });
                    let clone = Rc::clone(cached);
                    let success = clone.bind(&ctx.bindings);
                    *self = Self::Existing(clone);
                    success
                },
            }
        };
        if !success {
            let entry = ctx.texture_cache.entry(TextureCacheKey::error());
            let cached = entry.or_insert_with(move || {
                SharedTexture::new(Box::new(ErrorTexturePopulator))
            });
            let clone = Rc::clone(cached);
            assert!(
                clone.bind(&ctx.bindings),
                "Failed to bind error texture",
            );
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
            _ => false
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Texture {
    ins: TextureInstance,
    offset: (f32, f32),
    size: (f32, f32),
}

impl Texture {
    pub fn new<T, U>(populator: T) -> Self
    where
        T: Into<Box<U>>,
        U: PopulateTexture + 'static,
    {
        let populator = populator.into();
        let size = populator.get_known_size().unwrap_or((f32::NAN, f32::NAN));
        Self {
            ins: TextureInstance::Existing(SharedTexture::new(populator)),
            offset: (0.0, 0.0),
            size,
        }
    }

    pub fn size(&self) -> Option<(f32, f32)> {
        if !self.size.0.is_nan() {
            Some(self.size)
        } else {
            self.ins.image_size()
        }
    }

    pub fn from_alpha<T>(width: u16, height: u16, alignment: u16, pixels: T)
        -> Self
    where
        T: Into<Cow<'static, [u8]>>
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

    pub fn from_rgb<T>(width: u16, height: u16, alignment: u16, pixels: T)
        -> Self
    where
        T: Into<Cow<'static, [u8]>>
    {
        let pixels = pixels.into();
        assert_eq!(
            pixels.len(),
            PopulateTextureUtil::data_len(width, height, alignment, 3),
            "Invalid pixel data for given width/height/alignment",
        );
        Self::new(RGBTexturePopulator {
            width,
            height,
            alignment,
            pixels,
        })
    }

    pub fn from_rgba<T>(width: u16, height: u16, alignment: u16, pixels: T)
        -> Self
    where
        T: Into<Cow<'static, [u8]>>
    {
        let pixels = pixels.into();
        assert_eq!(
            pixels.len(),
            PopulateTextureUtil::data_len(width, height, alignment, 4),
            "Invalid pixel data for given width/height/alignment",
        );
        Self::new(RGBATexturePopulator {
            width,
            height,
            alignment,
            pixels,
        })
    }

    pub fn crop(self, x: f32, y: f32, height: f32, width: f32) -> Self {
        Self {
            ins: self.ins,
            offset: (self.offset.0 + x, self.offset.1 + y),
            size: (height, width),
        }
    }

    pub(super) fn bind(&mut self, ctx: &mut OpenGlContext)
        -> (f32, f32, f32, f32)
    {
        self.ins.bind(ctx);
        let size = self.ins.size().unwrap();
        size.get_crop_transform(
            self.offset.0,
            self.offset.1,
            self.size.0,
            self.size.1,
        )
    }

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

            &uvs[..]
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
