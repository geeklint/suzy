use std::cell::Cell;
use std::ffi::c_void;
use std::mem;
use std::ops::RangeBounds;
use std::path::Path;
use std::rc::Rc;

use super::OpenGlRenderPlatform as Gl;
use super::bindings::types::*;
use super::bindings::{
    CLAMP_TO_EDGE,
    COPY_READ_BUFFER,
    COPY_WRITE_BUFFER,
    DYNAMIC_DRAW,
    LINEAR,
    RGB,
    RGB8,
    RGBA,
    RGBA8,
    STATIC_DRAW,
    TEXTURE_2D,
    TEXTURE_MAG_FILTER,
    TEXTURE_MIN_FILTER,
    TEXTURE_WRAP_S,
    TEXTURE_WRAP_T,
    UNSIGNED_BYTE,
};

macro_rules! gl_object {
    ($name:ident, $create:ident, $delete:ident) => {
        #[derive(Clone, Debug, PartialEq)]
        struct $name {
            pub(crate) id: GLuint,
        }
        gl_object! {impl $name, $create, $delete}
    };
    (pub $name:ident, $create:ident, $delete:ident) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            pub(crate) id: GLuint,
        }
        gl_object! {impl $name, $create, $delete}
    };
    (impl $name:ident, $create:ident, $delete:ident) => {
        impl $name {
            pub fn new() -> Self {
                let mut id = 0;
                Gl::global(|gl| unsafe {
                    gl.$create(1, &mut id as *mut _);
                });
                Self { id }
            }
        }
        impl Drop for $name {
            fn drop(&mut self) {
                Gl::global(|gl| unsafe {
                    gl.$delete(1, &self.id as *const _);
                });
            }
        }
    };
}

gl_object! { pub VertexArrayObject, GenVertexArrays, DeleteVertexArrays }
gl_object! { BufferData, GenBuffers, DeleteBuffers }
gl_object! { TextureData, GenTextures, DeleteTextures }

pub struct Buffer<T> {
    obj: Rc<BufferData>,
    target: GLenum,
    dyn_draw: bool,
    len: usize,
    _data: std::marker::PhantomData<[T]>,
}

impl<T> Buffer<T> {
    pub fn new(target: GLenum, dyn_draw: bool, len: usize) -> Self {
        let obj = Rc::new(BufferData::new());
        Gl::global(|gl| unsafe {
            gl.BindBuffer(target, obj.id);
            gl.BufferData(
                target,
                (len * mem::size_of::<T>()) as GLsizeiptr,
                std::ptr::null(),
                if dyn_draw { DYNAMIC_DRAW } else { STATIC_DRAW },
            );
        });
        Self {
            obj: obj,
            target,
            dyn_draw,
            len,
            _data: std::marker::PhantomData,
        }
    }

    fn id(&self) -> GLuint { self.obj.id }

    pub fn len(&self) -> usize { self.len }

    pub fn bind(&self) {
        Gl::global(|gl| unsafe {
            gl.BindBuffer(self.target, self.id());
        });
    }

    pub fn set_data(&mut self, data: &[T]) {
        Gl::global(|gl| unsafe {
            gl.BindBuffer(self.target, self.id());
            if self.len == data.len() {
                gl.BufferSubData(
                    self.target,
                    0,
                    (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                    data.as_ptr() as *const c_void,
                );
            } else {
                gl.BufferData(
                    self.target,
                    (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                    data.as_ptr() as *const c_void,
                    if self.dyn_draw {
                        DYNAMIC_DRAW
                    } else {
                        STATIC_DRAW
                    },
                );
            }
        });
        self.len = data.len();
    }
}

impl<T> Clone for Buffer<T> {
    fn clone(&self) -> Self {
        if !self.dyn_draw {
            Self {
                obj: self.obj.clone(),
                target: self.target,
                dyn_draw: self.dyn_draw,
                len: self.len,
                _data: std::marker::PhantomData,
            }
        } else {
            let clone = Self::new(self.target, true, self.len);
            Gl::global(|gl| unsafe {
                gl.BindBuffer(COPY_READ_BUFFER, self.id());
                gl.BindBuffer(COPY_WRITE_BUFFER, clone.id());
                gl.CopyBufferSubData(
                    COPY_READ_BUFFER,
                    COPY_WRITE_BUFFER,
                    0, 0,
                    (self.len * mem::size_of::<T>()) as GLsizeiptr,
                );
            });
            clone
        }
    }
}

#[derive(Debug)]
pub struct TextureBuilder {
    id: TextureData,
    width: GLsizei,
    height: GLsizei,
    scale: [f32; 2],
}

impl TextureBuilder {
    pub fn create(width: GLsizei, height: GLsizei) -> Self {
        Self::create_custom(RGBA8, width, height)
    }

    pub fn create_opaque(width: GLsizei, height: GLsizei) -> Self {
        Self::create_custom(RGB8, width, height)
    }

    pub fn create_custom(
        format: GLenum,
        width: GLsizei,
        height: GLsizei,
    ) -> Self {
        let width_pot = (width as u32).next_power_of_two() as GLsizei;
        let height_pot = (height as u32).next_power_of_two() as GLsizei;
        let scale = [
            (width_pot as f32) / (width as f32),
            (height_pot as f32) / (height as f32),
        ];
        let id = TextureData::new();
        Gl::global(|gl| unsafe {
            gl.BindTexture(TEXTURE_2D, id.id);
            gl.TexImage2D(
                TEXTURE_2D,
                0,
                format as GLint,
                width_pot, height_pot,
                0,
                RGBA, UNSIGNED_BYTE,
                std::ptr::null(),
            );
        });
        Self { id, width: width_pot, height: height_pot, scale }
    }

    pub fn sub_image(
        &mut self,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *const c_void,
    ) {
        Gl::global(|gl| unsafe {
            gl.BindTexture(TEXTURE_2D, self.id.id);
            gl.TexSubImage2D(
                TEXTURE_2D,
                0,
                xoffset, yoffset,
                width, height,
                format, type_,
                pixels,
            );
        });
    }

    pub fn build(self) -> Texture {
        Gl::global(|gl| unsafe {
            gl.BindTexture(TEXTURE_2D, self.id.id);
            gl.TexParameteri(
                TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as GLint
            );
            gl.TexParameteri(
                TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as GLint
            );
            gl.TexParameteri(
                TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as GLint
            );
            gl.TexParameteri(
                TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as GLint
            );
            gl.GenerateMipmap(TEXTURE_2D);
        });
        Texture {
            obj: self.id,
            size: [self.width as f32, self.height as f32],
            offset: [0.0, 0.0],
            scale: self.scale,
        }
    }
}

impl Default for TextureBuilder {
    fn default() -> Self {
        let data: [u8; 12] = [0xff; 12];
        let mut builder = Self::create_opaque(2, 2);
        builder.sub_image(
            0, 0,
            2, 2,
            RGB, UNSIGNED_BYTE,
            data.as_ptr() as *const _,
        );
        builder
    }
}

pub type TextureLoadResult = Result<Texture, Box<dyn std::error::Error>>;
pub type TextureLoader = Option<fn(&Path) -> TextureLoadResult>;

thread_local! {
    static DEFAULT_TEXTURE: Texture = TextureBuilder::default().build();
    static TEXTURE_LOADER: Cell<TextureLoader> = Cell::new(None);
}

#[derive(Debug, Clone)]
pub struct Texture {
    obj: TextureData,
    pub(crate) size: [f32; 2],
    pub(crate) offset: [f32; 2],
    pub(crate) scale: [f32; 2],
}

impl Texture {
    pub fn width(&self) -> f32 { self.size[0] }
    pub fn height(&self) -> f32 { self.size[1] }

    pub(crate) fn bind(&self, gl: &super::Gl) {
        unsafe {
            gl.BindTexture(TEXTURE_2D, self.obj.id);
        }
    }

    pub fn crop<T, U>(&self, x: T, y: U) -> Self
    where
        T: RangeBounds<f32>,
        U: RangeBounds<f32>,
    {
        let _unused = (x, y);
        todo!()
    }

    pub fn set_loader(loader: TextureLoader) {
        TEXTURE_LOADER.with(|cell| cell.set(loader));
    }

    pub fn load<P: AsRef<Path>>(path: P) -> TextureLoadResult {
        let loader = TEXTURE_LOADER.with(|cell| cell.get());
        if let Some(load_fn) = loader {
            (load_fn)(path.as_ref())
        } else {
            Err(Box::new(TextureLoaderUnassigned))
        }
    }
}

impl PartialEq for Texture {
    fn eq(&self, other: &Texture) -> bool {
        self.obj == other.obj
            && self.offset == other.offset
            && self.scale == other.scale
    }
}

impl Default for Texture {
    fn default() -> Self {
        DEFAULT_TEXTURE.with(|tex| tex.clone())
    }
}

#[derive(Debug)]
pub struct TextureLoaderUnassigned;

impl std::error::Error for TextureLoaderUnassigned {}

impl std::fmt::Display for TextureLoaderUnassigned {
    fn fmt(&self, f: &mut std::fmt::Formatter)
        -> Result<(), std::fmt::Error>
    {
        let msg = concat!(
            "Texture Loader was never assigned a value using",
            " Texture::set_loader, or was subsequently assigned None"
        );
        std::fmt::Display::fmt(msg, f)
    }
}

