/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::convert::{TryFrom, TryInto};

#[derive(Clone, Copy, Debug)]
pub struct Vertex<Uv> {
    pub xy: [f32; 2],
    pub uv: [Uv; 2],
    pub color: [u8; 4],
    pub config: [u8; 4],
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

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
    fn to_f32(self) -> f32;
    fn try_from_f32(value: f32) -> Option<Self>;
    fn push(vv: &mut VertexVec, vertex: Vertex<Self>) -> u16;
}

impl UvType for f32 {
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
}
