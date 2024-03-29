/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::os::raw::{c_char, c_int, c_uchar, c_uint, c_void};

#[allow(non_camel_case_types)]
#[repr(C)]
pub(super) struct osmesa_context {
    _unused: [u8; 0],
}

pub(super) type OsMesaContext = *mut osmesa_context;

#[link(name = "OSMesa")]
extern "C" {
    pub(super) fn OSMesaCreateContext(
        format: c_uint,
        sharelist: OsMesaContext,
    ) -> OsMesaContext;

    pub(super) fn OSMesaDestroyContext(ctx: OsMesaContext);

    pub(super) fn OSMesaMakeCurrent(
        ctx: OsMesaContext,
        buffer: *mut c_void,
        type_: c_uint,
        width: c_int,
        height: c_int,
    ) -> c_uchar;

    pub(super) fn OSMesaGetProcAddress(
        funcName: *const c_char,
    ) -> *const c_void;
}
