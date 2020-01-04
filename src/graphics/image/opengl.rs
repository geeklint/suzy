use std::ffi::c_void;
use std::rc::Rc;

use gl::types::*;

use crate::graphics::{Graphic, DrawContext};
use crate::dims::{Dim, Rect, SimpleRect, SimplePadding2d, Padding2d};
use crate::math::Color;

#[derive(Debug)]
struct TextureData {
    id: GLuint,
}

impl TextureData {
    unsafe fn create(format: GLenum, width: GLsizei, height: GLsizei)
        -> Rc<Self>
    {
        let mut id = 0;
        gl::GenTextures(1, &mut id as *mut _);
        gl::BindTexture(gl::TEXTURE_2D, id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            format as GLint,
            width, height,
            0,
            gl::RGBA, gl::UNSIGNED_BYTE,
            std::ptr::null(),
        );
        Rc::new(Self { id })
    }

    unsafe fn sub_image(
        &self,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *const c_void,
    ) {
        gl::BindTexture(gl::TEXTURE_2D, self.id);
        gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0,
            xoffset, yoffset,
            width, height,
            format, type_,
            pixels,
        );
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
}

impl Drop for TextureData {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id as *const _);
        }
    }
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

    pub unsafe fn create(
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *const c_void,
    ) -> Self {
        Self::create_custom(
            gl::RGBA8,
            width, height,
            format, type_,
            pixels
        )
    }

    pub unsafe fn create_opaque(
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *const c_void,
    ) -> Self {
        Self::create_custom(
            gl::RGB8,
            width, height,
            format, type_,
            pixels
        )
    }

    pub unsafe fn create_gray(
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *const c_void,
    ) -> Self {
        Self::create_custom(
            gl::R8,
            width, height,
            format, type_,
            pixels
        )
    }

    pub unsafe fn create_custom(
        internalformat: GLenum,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *const c_void,
    ) -> Self {
        let width_pot = (width as u32).next_power_of_two() as GLsizei;
        let height_pot = (height as u32).next_power_of_two() as GLsizei;
        let obj;
        unsafe {
            obj = TextureData::create(
                internalformat,
                width_pot, height_pot,
            );
            obj.sub_image(
                0, 0, 
                width, height,
                format, type_,
                pixels,
            );
        }
        let scale = [
            (width as f32) / (width_pot as f32),
            (height as f32) / (height_pot as f32),
        ];
        Self {
            id: obj.id,
            _obj: obj,
            size: [width as f32, height as f32],
            offset: [0.0, 0.0],
            scale,
        }
    }

    pub unsafe fn small(color: Color) -> Self {
        let (r, g, b, a) = color.rgba();
        let data: [GLfloat; 16] = [
            r, g, b, a,
            r, g, b, a,
            r, g, b, a,
            r, g, b, a,
        ];
        Self::create(
            2, 2,
            gl::RGBA, gl::FLOAT,
            data.as_ptr() as *const c_void,
        )
    }

    pub unsafe fn tiny() -> Self {
        let data: [GLfloat; 4] = [1.0; 4];
        Self::create_gray(
            2, 2,
            gl::RED, gl::FLOAT,
            data.as_ptr() as *const c_void,
        )
    }
}

pub struct SlicedImage {
    dirty: std::cell::Cell<bool>,
    texture: Texture,
    rect: SimpleRect,
    padding: SimplePadding2d,
    vbo: GLuint,
    tbo: GLuint,
    ebo: GLuint,
}

static SLICED_INDICES: [u8; 18 * 3] = [
    0, 4, 11,
    4, 12, 11,
    4, 5, 12,
    5, 13, 12,
    5, 1, 13,
    1, 6, 13,
    11, 12, 10,
    12, 15, 10,
    12, 13, 15,
    13, 14, 15,
    13, 6, 14,
    6, 7, 14,
    10, 15, 3,
    15, 9, 3,
    15, 14, 9,
    14, 8, 9,
    14, 7, 8,
    7, 2, 8,
];

impl SlicedImage {
    pub fn create<R, P>(texture: Texture, rect: R, padding: P) -> Self
    where
        R: Into<SimpleRect>,
        P: Into<SimplePadding2d>,
    {
        let mut buffers = [0; 3];
        unsafe {
            gl::GenBuffers(3, buffers.as_mut_ptr());
        }
        let image = SlicedImage {
            dirty: std::cell::Cell::new(true),
            texture,
            rect: rect.into(),
            padding: padding.into(),
            vbo: buffers[0],
            tbo: buffers[1],
            ebo: buffers[2],
        };
        image.init_vertices();
        image.calc_uvs();
        image.gen_indices();
        image
    }

    fn init_vertices(&self) {
        let vertices: [GLfloat; 32] = [0.0; 32];
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(&vertices) as GLsizeiptr,
                vertices.as_ptr() as *const c_void,
                gl::DYNAMIC_DRAW,
            );
        }
    }

    fn gen_indices(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                std::mem::size_of_val(&SLICED_INDICES) as GLsizeiptr,
                SLICED_INDICES.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
        }
    }

    fn calc_uvs(&self) {
        let left = self.padding.left() / self.texture.width();
        let right = 1.0 - (self.padding.right() / self.texture.width());
        let bottom = self.padding.bottom() / self.texture.height();
        let top = 1.0 - (self.padding.top() / self.texture.height());
        let uvs: [GLfloat; 32] = [
            0.0, 0.0,
            1.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
            left, 0.0,
            right, 0.0,
            1.0, bottom,
            1.0, top,
            right, 1.0,
            left, 1.0,
            0.0, top,
            0.0, bottom,
            left, bottom,
            right, bottom,
            right, top,
            left, top,
        ];
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.tbo);
            gl::BufferData(
                gl::ARRAY_BUFFER, 
                std::mem::size_of_val(&uvs) as GLsizeiptr,
                uvs.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
        }
    }

    fn update(&self) {
        let mut inner = SimpleRect::default();
        inner.set_fill(&self.rect, &self.padding);
        let vertices: [GLfloat; 32] = [
            // outer corners
            self.rect.left(), self.rect.bottom(),
            self.rect.right(), self.rect.bottom(),
            self.rect.right(), self.rect.top(),
            self.rect.left(), self.rect.top(),
            // bottom edge
            inner.left(), self.rect.bottom(),
            inner.right(), self.rect.bottom(),
            // right edge
            self.rect.right(), inner.bottom(),
            self.rect.right(), inner.top(),
            // top edge
            inner.right(), self.rect.top(),
            inner.left(), self.rect.top(),
            // left edge
            self.rect.left(), inner.top(),
            self.rect.left(), inner.bottom(),
            // inner corners
            inner.left(), inner.bottom(),
            inner.right(), inner.bottom(),
            inner.right(), inner.top(),
            inner.left(), inner.top(),
        ];
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                std::mem::size_of_val(&vertices) as GLsizeiptr,
                vertices.as_ptr() as *const c_void,
            );
        }
        self.dirty.set(false);
    }
}

impl Rect for SlicedImage {
    fn x(&self) -> Dim { self.rect.x() }
    fn y(&self) -> Dim { self.rect.y() }

    fn x_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) {
        self.rect.x_mut(f);
        self.dirty.set(true);
    }

    fn y_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) {
        self.rect.y_mut(f);
        self.dirty.set(true);
    }
}

impl Graphic for SlicedImage {
    fn draw(&self, ctx: &mut DrawContext) {
        if self.dirty.get() {
            self.update();
        }
        DrawContext::push(ctx).use_texture(&self.texture);
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                0,
                std::ptr::null(),
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, self.tbo);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                0,
                std::ptr::null(),
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::DrawElements(
                gl::TRIANGLES,
                SLICED_INDICES.len() as GLsizei,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
        }
        DrawContext::pop(ctx);
    }
}
