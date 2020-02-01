use std::cell::Cell;
use std::ffi::c_void;
use std::mem;
use std::ops::RangeBounds;
use std::path::Path;
use std::rc::Rc;

use gl::types::*;

macro_rules! gl_object {
    ($name:ident, $create:ident, $delete:ident) => {
        #[derive(Debug)]
        struct $name {
            pub(crate) id: GLuint,
        }
        gl_object! {impl $name, $create, $delete}
    };
    (pub $name:ident, $create:ident, $delete:ident) => {
        #[derive(Debug)]
        pub struct $name {
            pub(crate) id: GLuint,
        }
        gl_object! {impl $name, $create, $delete}
    };
    (impl $name:ident, $create:ident, $delete:ident) => {
        impl $name {
            pub fn new() -> Self {
                let mut id = 0;
                unsafe { gl::$create(1, &mut id as *mut _) };
                Self { id }
            }
        }
        impl Drop for $name {
            fn drop(&mut self) {
                unsafe { gl::$delete(1, &self.id as *const _) };
            }
        }
    };
}

gl_object! { pub VertexArrayObject, GenVertexArrays, DeleteVertexArrays }
gl_object! { BufferData, GenBuffers, DeleteBuffers }
gl_object! { TextureData, GenTextures, DeleteTextures }

pub struct Buffer<T> {
    obj: Rc<BufferData>,
    id: GLuint,
    target: GLenum,
    dyn_draw: bool,
    len: usize,
    _data: std::marker::PhantomData<[T]>,
}

impl<T> Buffer<T> {
    pub fn new(target: GLenum, dyn_draw: bool, len: usize) -> Self {
        let obj = BufferData::new();
        unsafe {
            gl::BindBuffer(target, obj.id);
            gl::BufferData(
                target,
                (len * mem::size_of::<T>()) as GLsizeiptr,
                std::ptr::null(),
                if dyn_draw { gl::DYNAMIC_DRAW } else { gl::STATIC_DRAW },
            );
        }
        Self {
            id: obj.id,
            obj: Rc::new(obj),
            target,
            dyn_draw,
            len,
            _data: std::marker::PhantomData,
        }
    }

    pub fn len(&self) -> usize { self.len }

    pub unsafe fn bind(&self) {
        gl::BindBuffer(self.target, self.id);
    }

    pub fn set_data(&mut self, data: &[T]) {
        assert_eq!(data.len(), self.len);
        unsafe {
            gl::BindBuffer(self.target, self.id);
            gl::BufferSubData(
                self.target,
                0,
                (self.len * mem::size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as *const c_void,
            );
        }
    }
}

impl<T> Clone for Buffer<T> {
    fn clone(&self) -> Self {
        if !self.dyn_draw {
            Self {
                id: self.id,
                obj: Rc::clone(&self.obj),
                target: self.target,
                dyn_draw: self.dyn_draw,
                len: self.len,
                _data: std::marker::PhantomData,
            }
        } else {
            let mut clone = Self::new(self.target, true, self.len);
            unsafe {
                gl::BindBuffer(gl::COPY_READ_BUFFER, self.id);
                gl::BindBuffer(gl::COPY_WRITE_BUFFER, clone.id);
                gl::CopyBufferSubData(
                    gl::COPY_READ_BUFFER,
                    gl::COPY_WRITE_BUFFER,
                    0, 0,
                    (self.len * mem::size_of::<T>()) as GLsizeiptr,
                );
            }
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
        Self::create_custom(gl::RGBA8, width, height)
    }

    pub fn create_opaque(width: GLsizei, height: GLsizei) -> Self {
        Self::create_custom(gl::RGB8, width, height)
    }

    pub fn create_custom(format: GLenum, width: GLsizei, height: GLsizei)
        -> Self
    {
        let width_pot = (width as u32).next_power_of_two() as GLsizei;
        let height_pot = (height as u32).next_power_of_two() as GLsizei;
        let scale = [
            (width as f32) / (width_pot as f32),
            (height as f32) / (height_pot as f32),
        ];
        let id = TextureData::new();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, id.id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                format as GLint,
                width_pot, height_pot,
                0,
                gl::RGBA, gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
        }
        Self { id, width, height, scale }
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
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id.id);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                xoffset, yoffset,
                width, height,
                format, type_,
                pixels,
            );
        }
    }

    pub fn build(self) -> Texture {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id.id);
            gl::TexParameteri(
                gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint
            );
            gl::TexParameteri(
                gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint
            );
            gl::TexParameteri(
                gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint
            );
            gl::TexParameteri(
                gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        Texture {
            id: self.id.id,
            size: [self.width as f32, self.height as f32],
            offset: [0.0, 0.0],
            scale: self.scale,
            _obj: Rc::new(self.id),
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
            gl::RGB, gl::UNSIGNED_BYTE,
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
    _obj: Rc<TextureData>,
    pub(crate) id: GLuint,
    pub(crate) size: [f32; 2],
    pub(crate) offset: [f32; 2],
    pub(crate) scale: [f32; 2],
}

impl Texture {
    pub fn width(&self) -> f32 { self.size[0] }
    pub fn height(&self) -> f32 { self.size[1] }

    pub fn crop<T, U>(&self, x: T, y: U) -> Self
    where
        T: RangeBounds<f32>,
        U: RangeBounds<f32>,
    {
        todo!()
    }

    pub fn set_loader(loader: TextureLoader) {
        TEXTURE_LOADER.with(|cell| cell.set(loader));
    }

    pub fn load<P: AsRef<Path>>(path: &P) -> TextureLoadResult {
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
        self.id == other.id
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

