/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{
    convert::{TryFrom, TryInto},
    ffi::c_void,
    mem::size_of,
};

use crate::{
    platforms::opengl::opengl_bindings::types::{GLenum, GLsizei, GLsizeiptr},
    units::QuantizeU8,
};

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct VertexConfig(pub [u8; 4]);

#[derive(Clone, Copy, Debug, Default)]
pub struct Vertex<Uv> {
    pub xy: [f32; 2],
    pub uv: [Uv; 2],
    pub color: [u8; 4],
    pub config: VertexConfig,
    pub smoothing: f32,
}

impl<Uv> Vertex<Uv> {
    fn normalize(self) -> Vertex<f32>
    where
        Uv: UvType,
    {
        let [u, v] = self.uv;
        Vertex {
            xy: self.xy,
            uv: [u.to_f32(), v.to_f32()],
            color: self.color,
            config: self.config,
            smoothing: self.smoothing,
        }
    }
}

impl Default for VertexConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl VertexConfig {
    #[must_use]
    pub fn new() -> Self {
        Self([128, 255, 0, 0])
    }

    // The next two functions, along with the higher-precision `smoothing` in the
    // vertex attributes are used as parameters for a function in the shader that
    // allows adjusting the alpha value, mainly useful for rounded corners and
    // the signed distance fields we use for text.
    //
    // The `base` value is an offset that describes the intersection with the
    // x-axis, such that input alpha values less than `base` are clamped to
    // zero. The range is between -1.0 (exclusive) and 1.0 (inclusive). Negative
    // values are useful, however as the farthest from the normal range, -1.0
    // seemed the least useful and we want to be able to precisely represent
    // 0.0, so the quantization is done in terms of 256 instead of 255.
    //
    // The `peak` value is the x-value after which the line inverts and the
    // alpha starts to decrease again, which allows effects like hollow
    // outlines. This has a range of 0.0 to 1.0 (both inclusive) so standard
    // quantization is used.
    //
    // The smoothing value is the slope of the line, typically for the SDF text
    // this is going to be relatively large in order to represent a defined edge
    // (infinite would mean full aliasing), but can be smaller for soft shadows
    // or glows. If smoothing is negative, the alpha is inverted.
    //
    // The identity function has a base of 0.0, a peak of 1.0, and a slope of
    // 1.0. This is used for rendering normal images with an alpha channel where
    // we want the output unchanged.

    #[must_use]
    pub fn alpha_base(self, base: f32) -> Self {
        const QUANT_ADJ: f32 = 256.0 / 255.0;
        let Self([_, y, z, w]) = self;
        // (1.0)..(-1.0) => (0.0)..(1.0)
        let offset = (-base * 0.5) + 0.5;
        // (0.0)..(1.0) => (0)..(256)
        let x = (offset * QUANT_ADJ).quantize_u8();
        Self([x, y, z, w])
    }

    #[must_use]
    pub fn alpha_peak(self, peak: f32) -> Self {
        let Self([x, _, z, w]) = self;
        let y = peak.quantize_u8();
        Self([x, y, z, w])
    }

    #[must_use]
    pub fn vector(self, x_inside: bool, y_inside: bool) -> Self {
        let Self([x, y, _, w]) = self;
        let dx = if x_inside { 0 } else { 64 };
        let dy = if y_inside { 0 } else { 128 };
        let z = dx + dy;
        Self([x, y, z, w])
    }
}

#[derive(Debug)]
pub enum VertexVec {
    U16(Vec<Vertex<u16>>),
    F32(Vec<Vertex<f32>>),
}

impl Default for VertexVec {
    fn default() -> Self {
        Self::U16(Vec::new())
    }
}

impl VertexVec {
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            VertexVec::U16(vec) => vec.len(),
            VertexVec::F32(vec) => vec.len(),
        }
    }

    #[must_use]
    pub fn len_u16(&self) -> u16 {
        self.len().try_into().expect(
            "the number of vertices in a batch should be less than 2^16",
        )
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn reserve(&mut self, additional: usize) {
        match self {
            VertexVec::U16(vec) => vec.reserve(additional),
            VertexVec::F32(vec) => vec.reserve(additional),
        }
    }

    #[must_use]
    pub fn data_ptr(&self) -> *const std::ffi::c_void {
        match self {
            VertexVec::U16(vec) => vec.as_ptr().cast(),
            VertexVec::F32(vec) => vec.as_ptr().cast(),
        }
    }

    #[must_use]
    pub fn data_size(&self) -> GLsizeiptr {
        GLsizeiptr::try_from(match self {
            VertexVec::U16(vec) => vec.len() * size_of::<Vertex<u16>>(),
            VertexVec::F32(vec) => vec.len() * size_of::<Vertex<f32>>(),
        })
        .expect("vertex buffer should fit in GLsizei")
    }

    #[must_use]
    pub fn can_add(&self, num_vertices: u16) -> bool {
        self.len()
            .checked_add(num_vertices.into())
            .and_then(|len| u16::try_from(len).ok())
            .is_some()
    }

    pub fn push<Uv: UvType>(&mut self, vertex: Vertex<Uv>) -> u16 {
        Uv::push(self, vertex)
    }

    fn normalize(&mut self) -> &mut Vec<Vertex<f32>> {
        loop {
            match self {
                VertexVec::F32(vec) => break vec,
                VertexVec::U16(vec) => {
                    let new_vec = std::mem::take(vec)
                        .into_iter()
                        .map(Vertex::normalize)
                        .collect();
                    *self = Self::F32(new_vec);
                }
            }
        }
    }
}

pub trait UvType: Sized + Copy + Default {
    fn gl_type() -> GLenum;
    fn to_f32(self) -> f32;
    fn try_from_f32(value: f32) -> Option<Self>;
    fn push(vv: &mut VertexVec, vertex: Vertex<Self>) -> u16;
}

impl UvType for f32 {
    fn gl_type() -> GLenum {
        crate::platforms::opengl::opengl_bindings::FLOAT
    }

    fn to_f32(self) -> f32 {
        self
    }

    fn try_from_f32(value: f32) -> Option<Self> {
        Some(value)
    }

    fn push(vv: &mut VertexVec, vertex: Vertex<Self>) -> u16 {
        let vec = vv.normalize();
        let len = vec.len();
        vec.push(vertex);
        len.try_into().expect("exceeded 2**16 verts in a batch")
    }
}

impl UvType for u16 {
    fn gl_type() -> GLenum {
        crate::platforms::opengl::opengl_bindings::UNSIGNED_SHORT
    }

    fn to_f32(self) -> f32 {
        self.into()
    }

    fn try_from_f32(value: f32) -> Option<Self> {
        let nearest = value as u16;
        (f32::from(nearest) == value).then_some(nearest)
    }

    fn push(vv: &mut VertexVec, vertex: Vertex<Self>) -> u16 {
        match vv {
            VertexVec::U16(vec) => {
                let len = vec.len();
                vec.push(vertex);
                len.try_into().expect("exceeded 2**16 verts in a batch")
            }
            VertexVec::F32(vec) => {
                let len = vec.len();
                vec.push(vertex.normalize());
                len.try_into().expect("exceeded 2**16 verts in a batch")
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UvRectValues<T> {
    pub left: T,
    pub right: T,
    pub bottom: T,
    pub top: T,
}

#[derive(Clone, Copy, Debug)]
pub enum UvRect {
    F32(UvRectValues<f32>),
    U16(UvRectValues<u16>),
    SolidColor(u16, u16),
}

pub(super) struct OffsetInfo {
    pub xy: *const c_void,
    pub uv: *const c_void,
    pub color: *const c_void,
    pub config: *const c_void,
    pub smoothing: *const c_void,
    pub stride: GLsizei,
    pub uv_type: GLenum,
}

impl OffsetInfo {
    pub fn for_vertex_vec(vec: &VertexVec) -> Self {
        match vec {
            VertexVec::U16(_) => Self::for_uv_type::<u16>(),
            VertexVec::F32(_) => Self::for_uv_type::<f32>(),
        }
    }

    pub fn for_uv_type<Uv: UvType>() -> Self {
        macro_rules! offset_of {
            ($Container:ty, $field:ident) => {{
                offset_as_ptr(std::mem::offset_of!($Container, $field))
            }};
        }
        let xy = offset_of!(Vertex<Uv>, xy);
        let uv = offset_of!(Vertex<Uv>, uv);
        let color = offset_of!(Vertex<Uv>, color);
        let config = offset_of!(Vertex<Uv>, config);
        let smoothing = offset_of!(Vertex<Uv>, smoothing);
        let stride = size_of::<Vertex<Uv>>().try_into().expect(
            "vertex struct should have a size small enough to fit in GLsizei",
        );
        let uv_type = Uv::gl_type();
        Self {
            xy,
            uv,
            color,
            config,
            smoothing,
            stride,
            uv_type,
        }
    }
}

fn offset_as_ptr(offset: usize) -> *const c_void {
    let signed_offset = isize::try_from(offset)
        .expect("struct field offsets should fit within an isize");
    std::ptr::null::<c_void>().wrapping_offset(signed_offset)
}
