/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::{Index, IndexMut, Mul, MulAssign};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    data: [f32; 16],
}

impl Mat4 {
    pub const fn identity() -> Self {
        Self {
            data: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub const fn translate(x: f32, y: f32) -> Self {
        Self {
            data: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0, x, y, 0.0, 1.0,
            ],
        }
    }

    pub const fn scale(x: f32, y: f32) -> Self {
        Self {
            data: [
                x, 0.0, 0.0, 0.0, 0.0, y, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn rotate(radians: f32) -> Self {
        Self {
            data: [
                radians.cos(), radians.sin(), 0.0, 0.0,
                -radians.sin(), radians.cos(), 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }
}

impl AsRef<[f32; 16]> for Mat4 {
    fn as_ref(&self) -> &[f32; 16] {
        &self.data
    }
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl From<[f32; 16]> for Mat4 {
    fn from(data: [f32; 16]) -> Self {
        Self { data }
    }
}

impl<I, J> Index<(I, J)> for Mat4
where
    I: Into<usize>,
    J: Into<usize>,
{
    type Output = f32;

    fn index(&self, (row, col): (I, J)) -> &f32 {
        &self.data[col.into() * 4 + row.into()]
    }
}

impl<I, J> IndexMut<(I, J)> for Mat4
where
    I: Into<usize>,
    J: Into<usize>,
{
    fn index_mut(&mut self, (row, col): (I, J)) -> &mut f32 {
        &mut self.data[col.into() * 4 + row.into()]
    }
}

macro_rules! mul {
    ( $left:expr, $right:expr; $row:expr, $col:expr ) => {
        (
            $left[($row, 0u8)] * $right[(0u8, $col)]
            + $left[($row, 1u8)] * $right[(1u8, $col)]
            + $left[($row, 2u8)] * $right[(2u8, $col)]
            + $left[($row, 3u8)] * $right[(3u8, $col)]
        )
    };
}

impl Mul<&Mat4> for &Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: &Mat4) -> Mat4 {
        Mat4 {
            data: [
                mul!(self, rhs; 0u8, 0u8),
                mul!(self, rhs; 1u8, 0u8),
                mul!(self, rhs; 2u8, 0u8),
                mul!(self, rhs; 3u8, 0u8),
                mul!(self, rhs; 0u8, 1u8),
                mul!(self, rhs; 1u8, 1u8),
                mul!(self, rhs; 2u8, 1u8),
                mul!(self, rhs; 3u8, 1u8),
                mul!(self, rhs; 0u8, 2u8),
                mul!(self, rhs; 1u8, 2u8),
                mul!(self, rhs; 2u8, 2u8),
                mul!(self, rhs; 3u8, 2u8),
                mul!(self, rhs; 0u8, 3u8),
                mul!(self, rhs; 1u8, 3u8),
                mul!(self, rhs; 2u8, 3u8),
                mul!(self, rhs; 3u8, 3u8),
            ]
        }
    }
}

impl Mul for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Mat4 {
        #![allow(clippy::op_ref)]
        (&self) * (&rhs)
    }
}

impl Mul<&Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: &Mat4) -> Mat4 {
        #![allow(clippy::op_ref)]
        (&self) * rhs
    }
}

impl Mul<Mat4> for &Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Mat4 {
        #![allow(clippy::op_ref)]
        self * (&rhs)
    }
}

impl MulAssign for Mat4 {
    fn mul_assign(&mut self, rhs: Mat4) {
        #![allow(clippy::op_ref)]
        *self = &*self * rhs
    }
}

impl MulAssign<&Mat4> for Mat4 {
    fn mul_assign(&mut self, rhs: &Mat4) {
        #![allow(clippy::op_ref)]
        *self = &*self * rhs
    }
}

type Vec4 = (f32, f32, f32, f32);

macro_rules! mul_vec {
    ( $left:expr, $right:expr; $row:expr ) => {
        (
            $left[($row, 0u8)] * $right.0
            + $left[($row, 1u8)] * $right.1
            + $left[($row, 2u8)] * $right.2
            + $left[($row, 3u8)] * $right.3
        )
    }
}

impl Mul<&Vec4> for &Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: &Vec4) -> Vec4 {
        (
            mul_vec!(self, rhs; 0u8),
            mul_vec!(self, rhs; 1u8),
            mul_vec!(self, rhs; 2u8),
            mul_vec!(self, rhs; 3u8),
        )
    }
}

impl Mul<&Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: &Vec4) -> Vec4 {
        (
            mul_vec!(self, rhs; 0u8),
            mul_vec!(self, rhs; 1u8),
            mul_vec!(self, rhs; 2u8),
            mul_vec!(self, rhs; 3u8),
        )
    }
}

impl Mul<Vec4> for &Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Vec4 {
        (
            mul_vec!(self, rhs; 0u8),
            mul_vec!(self, rhs; 1u8),
            mul_vec!(self, rhs; 2u8),
            mul_vec!(self, rhs; 3u8),
        )
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Vec4 {
        (
            mul_vec!(self, rhs; 0u8),
            mul_vec!(self, rhs; 1u8),
            mul_vec!(self, rhs; 2u8),
            mul_vec!(self, rhs; 3u8),
        )
    }
}

#[cfg(test)]
mod test {
    // TODO: clippy is probably right
    #![allow(clippy::excessive_precision)]

    use super::*;

    const SOME_VEC: Vec4 = (
        0.4590671600603613,
        0.07089256298264268, 
        0.12435283851511914,
        0.42263442840089094,
    );

    const SOME_MAT: Mat4 = Mat4 {
        data: [
            0.31398639583501475, 0.6445062614061633, 0.2860791956082398,
            0.697080107788084, 0.29709876564325355, 0.26833200285392045,
            0.8025856253223312, 0.8036435486803184, 0.33477434202929435,
            0.2799811230631124, 0.5069393506047003, 0.5936022903429544,
            0.744318146423893, 0.6871507165508559, 0.2700660023092307,
            0.04199180074417164,
        ],
    };
    
    #[test]
    fn check_identity() {
        assert_eq!(SOME_VEC, Mat4::identity() * SOME_VEC);
        assert_eq!(SOME_MAT, Mat4::identity() * SOME_MAT);
    }

    const MAT_A: Mat4 = Mat4 {
        data: [
            2.0, 7.0, 7.0, 2.0, 6.0, 5.0, 9.0, 5.0,
            8.0, 6.0, 2.0, 4.0, 6.0, 9.0, 7.0, 6.0,
        ],
    };
    const MAT_B: Mat4 = Mat4 {
        data: [
            3.0, 3.0, 5.0, 8.0, 8.0, 3.0, 5.0, 3.0,
            8.0, 6.0, 5.0, 1.0, 9.0, 2.0, 7.0, 9.0,
        ],
    };
    const MAT_ANS: Mat4 = Mat4 {
        data: [
            112.0, 138.0, 114.0, 89.0, 92.0, 128.0, 114.0, 69.0,
            98.0, 125.0, 127.0, 72.0, 140.0, 196.0, 158.0, 110.0,
        ],
    };

    #[test]
    fn check_mat_mul() {
        assert_eq!(MAT_ANS, MAT_A * MAT_B)
    }

    #[test]
    fn check_rotate() {
        let result = Mat4::rotate(std::f32::consts::PI) * SOME_VEC;
        assert!(result.0 + SOME_VEC.0 <= f32::EPSILON);
        assert!(result.1 + SOME_VEC.1 <= f32::EPSILON);
        #[allow(clippy::float_cmp)]
        {
            assert_eq!(result.2, SOME_VEC.2);
            assert_eq!(result.3, SOME_VEC.3);
        }
    }
}
