/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::ffi::{CStr, CString};
use std::rc::{Rc, Weak};

use super::context::bindings::types::*;
use super::context::bindings::{
    ACTIVE_ATTRIBUTES, COMPILE_STATUS, FALSE, FRAGMENT_SHADER,
    INFO_LOG_LENGTH, LINK_STATUS, MAX_VERTEX_ATTRIBS, VERTEX_SHADER,
};
use super::OpenGlBindings;

macro_rules! info_log {
    ( $id:expr, $gl:expr, $fn_get_iv:ident, $fn_log:ident ) => {{
        let log_len = unsafe {
            let mut log_len = 0;
            $gl.$fn_get_iv($id, INFO_LOG_LENGTH, &mut log_len as *mut GLint);
            log_len as usize
        };
        let mut array = vec![0; log_len];
        let mut actual_len = 0;
        unsafe {
            $gl.$fn_log(
                $id,
                array.len() as GLsizei,
                &mut actual_len as *mut GLsizei,
                array.as_mut_ptr() as *mut GLchar,
            );
        }
        array.truncate((actual_len + 1) as usize);
        CStr::from_bytes_with_nul(&array).unwrap().to_owned()
    }};
}

struct ShaderObj<'a> {
    id: GLuint,
    gl: &'a OpenGlBindings,
}

impl<'a> Drop for ShaderObj<'a> {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
    }
}

fn compile_shader<'a>(
    gl: &'a OpenGlBindings,
    type_: GLenum,
    text: &[u8],
) -> Result<ShaderObj<'a>, CString> {
    let lines = [text.as_ptr()];
    let lengths = [text.len()];
    let (obj, success) = unsafe {
        let obj = ShaderObj {
            id: gl.CreateShader(type_),
            gl,
        };
        gl.ShaderSource(
            obj.id,
            lines.len() as GLsizei,
            lines.as_ptr() as *const *const GLchar,
            lengths.as_ptr() as *const GLint,
        );
        gl.CompileShader(obj.id);
        let mut success: GLint = 0;
        gl.GetShaderiv(obj.id, COMPILE_STATUS, &mut success as *mut GLint);
        (obj, success)
    };
    if success != FALSE as GLint {
        Ok(obj)
    } else {
        let log = info_log!(obj.id, gl, GetShaderiv, GetShaderInfoLog);
        Err(log)
    }
}

#[derive(Debug)]
pub enum ProgramCompileError {
    Vertex(CString),
    Fragment(CString),
    Link(CString),
}

struct ProgramObject {
    id: GLuint,
    gl: Weak<OpenGlBindings>,
}

impl Drop for ProgramObject {
    fn drop(&mut self) {
        // if we can't get the gl bindings here, it's probably
        // because the context went away, in which case
        // it's ok to "leak" the resource, it's already cleaned
        // up by the context going away
        if let Some(gl) = self.gl.upgrade() {
            unsafe {
                gl.DeleteProgram(self.id);
            }
        }
    }
}

fn compile_program(
    gl: &Rc<OpenGlBindings>,
    vert_text: &[u8],
    frag_text: &[u8],
) -> Result<ProgramObject, ProgramCompileError> {
    let vert = compile_shader(gl, VERTEX_SHADER, vert_text)
        .map_err(ProgramCompileError::Vertex)?;
    let frag = compile_shader(gl, FRAGMENT_SHADER, frag_text)
        .map_err(ProgramCompileError::Fragment)?;
    let (success, program) = unsafe {
        let program = ProgramObject {
            id: gl.CreateProgram(),
            gl: Rc::downgrade(gl),
        };
        gl.AttachShader(program.id, vert.id);
        gl.AttachShader(program.id, frag.id);
        gl.LinkProgram(program.id);
        let mut success: GLint = 0;
        gl.GetProgramiv(program.id, LINK_STATUS, &mut success as *mut GLint);
        let success = success != (FALSE as GLint);
        (success, program)
    };
    let result = if success {
        Ok(())
    } else {
        Err(ProgramCompileError::Link(info_log!(
            program.id,
            gl,
            GetProgramiv,
            GetProgramInfoLog
        )))
    };
    unsafe {
        gl.DetachShader(program.id, vert.id);
        gl.DetachShader(program.id, frag.id);
    }
    result.map(|()| program)
}

#[derive(Clone)]
pub struct ShaderProgram {
    _obj: Rc<ProgramObject>,
    program_id: GLuint,
    attrs: GLuint,
    total_attrs: GLuint,
}

#[derive(Clone, Copy, Debug)]
pub struct UniformLoc {
    id: GLint,
}

impl ShaderProgram {
    pub fn create(
        gl: &Rc<OpenGlBindings>,
        vert_text: &[u8],
        frag_text: &[u8],
    ) -> Result<Self, ProgramCompileError> {
        let obj = compile_program(gl, vert_text, frag_text)?;
        let (attrs, total_attrs) = unsafe {
            let mut attrs: GLint = 0;
            let mut total_attrs: GLint = 8;
            gl.GetProgramiv(
                obj.id,
                ACTIVE_ATTRIBUTES,
                &mut attrs as *mut GLint,
            );
            gl.GetIntegerv(MAX_VERTEX_ATTRIBS, &mut total_attrs as *mut GLint);
            (attrs as GLuint, total_attrs as GLuint)
        };
        let shader = ShaderProgram {
            program_id: obj.id,
            _obj: Rc::new(obj),
            attrs,
            total_attrs,
        };
        Ok(shader)
    }

    pub fn make_current(
        &self,
        gl: &OpenGlBindings,
        prev_attribs: Option<GLuint>,
    ) {
        let start = prev_attribs.unwrap_or(0);
        let end = prev_attribs.unwrap_or(self.total_attrs);
        unsafe {
            gl.UseProgram(self.program_id);
            for i in start..self.attrs {
                gl.EnableVertexAttribArray(i);
            }
            for i in self.attrs..end {
                gl.DisableVertexAttribArray(i);
            }
        }
    }

    pub fn uniform(&self, gl: &OpenGlBindings, name: &str) -> UniformLoc {
        let cname = CString::new(name).expect("Uniform name contained null");
        let id = unsafe {
            gl.GetUniformLocation(
                self.program_id,
                cname.as_ptr() as *const GLchar,
            )
        };
        debug_assert_ne!(id, -1, "Failed to get uniform {}", name);
        UniformLoc { id }
    }

    pub fn set_opaque(gl: &OpenGlBindings, loc: UniformLoc, value: GLuint) {
        unsafe { gl.Uniform1i(loc.id, value as GLint) };
    }

    pub fn set_vec4(
        gl: &OpenGlBindings,
        loc: UniformLoc,
        value: (GLfloat, GLfloat, GLfloat, GLfloat),
    ) {
        unsafe {
            gl.Uniform4f(loc.id, value.0, value.1, value.2, value.3);
        }
    }

    pub fn set_mat4(gl: &OpenGlBindings, loc: UniformLoc, value: &[GLfloat]) {
        debug_assert_eq!(value.len(), 16, "mat4 must have 16 elements!");
        unsafe {
            gl.UniformMatrix4fv(loc.id, 1, FALSE, value.as_ptr());
        }
    }
}
