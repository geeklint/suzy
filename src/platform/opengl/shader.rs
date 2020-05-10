use std::ffi::{CStr, CString};
use std::rc::Rc;

use super::OpenGlRenderPlatform as Gl;
use super::bindings::types::*;
use super::bindings::{
    ACTIVE_ATTRIBUTES,
    COMPILE_STATUS,
    FALSE,
    FRAGMENT_SHADER,
    INFO_LOG_LENGTH,
    LINK_STATUS,
    MAX_VERTEX_ATTRIBS,
    VERTEX_SHADER,
};

macro_rules! info_log {
    ( $id:expr, $fn_iv:ident, $fn_log:ident ) => {
        {
            let log_len = Gl::global(|gl| unsafe {
                let mut log_len = 0;
                gl.$fn_iv(
                    $id,
                    INFO_LOG_LENGTH,
                    &mut log_len as *mut GLint,
                );
                log_len as usize
            });
            let mut array = vec![0; log_len];
            let mut actual_len = 0;
            Gl::global(|gl| unsafe {
                gl.$fn_log(
                    $id,
                    array.len() as GLsizei,
                    &mut actual_len as *mut GLsizei,
                    array.as_mut_ptr() as *mut GLchar,
                );
            });
            array.truncate((actual_len + 1) as usize);
            CStr::from_bytes_with_nul(&array).unwrap().to_owned()
        }
    };
}

fn compile_shader(type_: GLenum, text: &[u8]) -> Result<GLuint, CString> {
    let lines = [text.as_ptr()];
    let lengths = [text.len()];
    let (id, success) = Gl::global(|gl| unsafe {
        let id = gl.CreateShader(type_);
        gl.ShaderSource(
            id,
            lines.len() as GLsizei,
            lines.as_ptr() as *const *const GLchar,
            lengths.as_ptr() as *const GLint,
        );
        gl.CompileShader(id);
        let mut success: GLint = 0;
        gl.GetShaderiv(id, COMPILE_STATUS, &mut success as *mut GLint);
        (id, success)
    });
    if success != FALSE as GLint {
        Ok(id)
    } else {
        let log = info_log!(id, GetShaderiv, GetShaderInfoLog);
        Gl::global(|gl| unsafe { gl.DeleteShader(id) });
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
}

impl Drop for ProgramObject {
    fn drop(&mut self) {
        // if we can't get the gl bindings here, it's probably
        // because the whole app is shutting down, in which case
        // it's ok to leak the resource, it'll get cleaned up
        // by the context getting disposed.
        Gl::try_global(|gl| unsafe { gl.DeleteProgram(self.id) });
    }
}

fn compile_program(vert_text: &[u8], frag_text: &[u8])
    -> Result<ProgramObject, ProgramCompileError>
{
    let vert_id = compile_shader(VERTEX_SHADER, vert_text)
        .map_err(ProgramCompileError::Vertex)?;
    let frag_id = compile_shader(FRAGMENT_SHADER, frag_text)
        .map_err(ProgramCompileError::Fragment)?;
    let (success, program) = Gl::global(|gl| unsafe {
        let program = ProgramObject {
            id: gl.CreateProgram(),
        };
        gl.AttachShader(program.id, vert_id);
        gl.AttachShader(program.id, frag_id);
        gl.LinkProgram(program.id);
        let mut success: GLint = 0;
        gl.GetProgramiv(program.id, LINK_STATUS, &mut success as *mut GLint);
        let success = success != (FALSE as GLint);
        (success, program)
    });
    let err = if success {
        Ok(())
    } else {
        Err(ProgramCompileError::Link(
            info_log!(program.id, GetProgramiv, GetProgramInfoLog)
        ))
    };
    Gl::global(|gl| unsafe {
        gl.DetachShader(program.id, vert_id);
        gl.DetachShader(program.id, frag_id);
        gl.DeleteShader(vert_id);
        gl.DeleteShader(frag_id);
    });
    err.map(|_| program)
}

#[derive(Clone)]
pub struct Shader {
    _obj: Rc<ProgramObject>,
    program_id: GLuint,
    attrs: GLint,
    total_attrs: GLint,
}

#[derive(Clone, Copy, Debug)]
pub struct UniformLoc {
    id: GLint,
}

impl Shader {
    pub fn create(vert_text: &[u8], frag_text: &[u8])
        -> Result<Self, ProgramCompileError>
    {
        let obj = compile_program(vert_text, frag_text)?;
        let (attrs, total_attrs) = Gl::global(|gl| unsafe {
            let mut attrs: GLint = 0;
            let mut total_attrs: GLint = 8;
            gl.GetProgramiv(
                obj.id,
                ACTIVE_ATTRIBUTES,
                &mut attrs as *mut GLint,
            );
            gl.GetIntegerv(
                MAX_VERTEX_ATTRIBS,
                &mut total_attrs as *mut GLint,
            );
            (attrs, total_attrs)
        });
        let shader = Shader {
            program_id: obj.id,
            _obj: Rc::new(obj),
            attrs,
            total_attrs,
        };
        Ok(shader)
    }

    pub fn make_current(&self) {
        Gl::global(|gl| unsafe {
            gl.UseProgram(self.program_id);
            for i in 0..self.attrs {
                gl.EnableVertexAttribArray(i as GLuint);
            }
            for i in self.attrs..self.total_attrs {
                gl.DisableVertexAttribArray(i as GLuint);
            }
        });
    }

    pub fn uniform(&self, name: &str) -> UniformLoc {
        let prog_id = self.program_id;
        let cname = CString::new(name)
            .expect("Uniform name contained null");
        let id = Gl::global(|gl| unsafe {
            gl.GetUniformLocation(
                prog_id,
                cname.as_ptr() as *const GLchar,
            )
        });
        debug_assert_ne!(id, -1, "Failed to get uniform {}", name);
        UniformLoc { id }
    }

    pub fn set_opaque(loc: UniformLoc, value: GLuint) {
        Gl::global(|gl| unsafe {
            gl.Uniform1i(loc.id, value as GLint);
        });
    }

    pub fn set_float(loc: UniformLoc, value: GLfloat) {
        Gl::global(|gl| unsafe {
            gl.Uniform1f(loc.id, value);
        });
    }

    pub fn set_vec2(loc: UniformLoc, value: (GLfloat, GLfloat)) {
        Gl::global(|gl| unsafe {
            gl.Uniform2f(loc.id, value.0, value.1);
        });
    }

    pub fn set_vec4(
        loc: UniformLoc,
        value: (GLfloat, GLfloat, GLfloat, GLfloat),
    ) {
        Gl::global(|gl| unsafe {
            gl.Uniform4f(
                loc.id,
                value.0, value.1, value.2, value.3,
            );
        });
    }

    pub fn set_mat4(loc: UniformLoc, value: &[GLfloat]) {
        debug_assert_eq!(value.len(), 16, "mat4 must have 16 elements!");
        Gl::global(|gl| unsafe {
            gl.UniformMatrix4fv(
                loc.id,
                1,
                FALSE,
                value.as_ptr(),
            );
        });
    }
}
