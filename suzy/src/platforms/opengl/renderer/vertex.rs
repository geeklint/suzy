/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{
    convert::{TryFrom, TryInto},
    mem::size_of,
};

use crate::platforms::opengl::opengl_bindings::types::GLenum;

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

    #[must_use]
    pub fn alpha_base(self, base: f32) -> Self {
        let Self([_, y, z, w]) = self;
        let magic = 255.0 / 2.0;
        let offset = (-magic * base + magic).clamp(0.0, 255.0).round();
        let x = offset as u8;
        Self([x, y, z, w])
    }

    #[must_use]
    pub fn alpha_base_by_odd(self, odd: bool) -> Self {
        // can't represent exactly 0.5 in a normalized u8
        // so we flip between 127 and 128 and let it interpolate
        // to 0.5 between vertices
        let Self([_, y, z, w]) = self;
        let x = if odd { 128 } else { 127 };
        Self([x, y, z, w])
    }

    #[must_use]
    pub fn alpha_peak(self, peak: f32) -> Self {
        let Self([x, _, z, w]) = self;
        let y = (peak * 255.0).clamp(0.0, 255.0).round() as u8;
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
    pub fn len(&self) -> usize {
        match self {
            VertexVec::U16(vec) => vec.len(),
            VertexVec::F32(vec) => vec.len(),
        }
    }

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

    pub fn data_ptr(&self) -> *const std::ffi::c_void {
        match self {
            VertexVec::U16(vec) => vec.as_ptr().cast(),
            VertexVec::F32(vec) => vec.as_ptr().cast(),
        }
    }

    pub fn data_size(
        &self,
    ) -> crate::platforms::opengl::opengl_bindings::types::GLsizeiptr {
        TryFrom::try_from(match self {
            VertexVec::U16(vec) => vec.len() * size_of::<Vertex<u16>>(),
            VertexVec::F32(vec) => vec.len() * size_of::<Vertex<f32>>(),
        })
        .expect("vertex buffer should fit in GLsizei")
    }

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
    pub xy: usize,
    pub uv: usize,
    pub color: usize,
    pub config: usize,
    pub smoothing: usize,
    pub stride: usize,
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
        let base_vertex = Vertex::<Uv>::default();
        let abs_xy = base_vertex.xy.as_ptr() as usize;
        let abs_uv = base_vertex.uv.as_ptr() as usize;
        let abs_color = base_vertex.color.as_ptr() as usize;
        let abs_config = base_vertex.config.0.as_ptr() as usize;
        let abs_smoothing = (&base_vertex.smoothing as *const f32) as usize;
        let abs_base = (&base_vertex as *const Vertex<Uv>) as usize;
        let xy = abs_xy - abs_base;
        let uv = abs_uv - abs_base;
        let color = abs_color - abs_base;
        let config = abs_config - abs_base;
        let smoothing = abs_smoothing - abs_base;
        let stride = size_of::<Vertex<Uv>>();
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
