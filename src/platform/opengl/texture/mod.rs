use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::bindings::{
    CLAMP_TO_EDGE,
    LINEAR,
    NEAREST,
    RGB,
    RGBA,
    TEXTURE_2D,
    TEXTURE_MAG_FILTER,
    TEXTURE_MIN_FILTER,
    TEXTURE_WRAP_S,
    TEXTURE_WRAP_T,
    UNSIGNED_BYTE,
};

mod populate;

pub use populate::*;

pub(super) type TextureCache = HashMap<TextureCacheKey, Rc<SharedTexture>>;

gl_object! { TextureData, GenTextures, DeleteTextures, 1 }

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
        let offset_x = (x / self.texture_width);
        let offset_y = (y / self.texture_height);
        let scale_x = self.texture_width / width;
        let scale_y = self.texture_height / height;
        (offset_x, offset_y, scale_x, scale_y)
    }
}

pub(super) struct SizedTexture {
    obj: TextureData,
    size: TextureSize,
}

impl SizedTexture {
    fn size(&self) -> Option<&TextureSize> {
        if self.obj.ready {
            Some(&self.size)
        } else {
            None
        }
    }
}

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

    fn bind(&self, gl: &Rc<OpenGlBindings>) {
        let borrow = self.obj.borrow_mut();
        let SizedTexture { obj, size } = borrow;
        let ready = obj.check_ready(gl);
        gl.BindTexture(TEXTURE_2D, obj.ids[0]);
        if !ready {
            *size = self.populator.populate(gl);
            obj.mark_ready();
        }
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
    fn from(path: &'static str) -> Self {
        Self { inner: KeyVariants::Path(path.into()) }
    }
}

impl From<String> for TextureCacheKey {
    fn from(string: String) -> Self {
        string.into::<PathBuf>().into()
    }
}

impl From<&'static str> for TextureCacheKey {
    fn from(string: &'static str) -> Self {
        string.as_ref::<Path>().into()
    }
}

#[derive(Clone)]
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
            Self::ToCache(_) => None,
        }
    }

    fn ensure_existing(&mut self, cache: &mut TextureCache)
        -> &Rc<SharedTexture>
    {
        if let Self::Existing(ref existing) = self {
            &existing
        } else {
            match std::mem::replace(self, Self::error()) {
                // TODO: figure out how to do this without unreachable!() ?
                Self::Existing(_) => unreachable!(),
                Self::ToCache(key, populator) => {
                    let entry = cache.entry(key);
                    let cached = entry.or_insert_with(move || {
                        SharedTexture::new(populator)
                    });
                    *self = Self::Existing(cached.clone())
                    if let Self::Existing(ref existing) = self {
                        &existing
                    } else {
                        unreachable!()
                    }
                },
            }
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

#[derive(Clone)]
pub struct Texture {
    ins: TextureInstance,
    offset: (f32, f32),
    size: (f32, f32),
}

impl Texture {
    pub fn new<T: Into<TexurePopulator>>(populator: T) -> Self {
        let populator = populator.into().0;
        let size = populator.get_known_size().unwrap_or((f32::NAN, f32::NAN));
        Self {
            ins: TextureInstance::Existing(SharedTexture::new(populator)),
            offset: (0.0, 0.0),
            size,
        }
    }

    pub fn from_alpha<T>(width: usize, height: usize, pixels: T)
    where
        T: Into<Cow<'static, [u8]>>>
    {
        let pixels = pixels.into();
        Self::new(|gl| {
            PopulateTextureUtil::populate_alpha(gl, width, height, &pixels)
        });
    }

    pub fn from_rgb<T>(width: usize, height: usize, pixels: T)
    where
        T: Into<Cow<'static, [u8]>>>
    {
        let pixels = pixels.into();
        Self::new(|gl| {
            PopulateTextureUtil::populate_rgb(gl, width, height, &pixels)
        });
    }

    pub fn from_rgba<T>(width: usize, height: usize, pixels: T)
    where
        T: Into<Cow<'static, [u8]>>>
    {
        let pixels = pixels.into();
        Self::new(|gl| {
            PopulateTextureUtil::populate_rgba(gl, width, height, &pixels)
        });
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
        self.ins.ensure_existing(&mut ctx.texture_cache).bind(&ctx.bindings);
        self.ins.size().unwrap().get_crop_transform(
            self.offset.0,
            self.offset.1,
            self.size.0,
            self.size.1,
        )
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
