use std::ffi::{CStr, CString};
use std::rc::Rc;

use gl::types::*;

macro_rules! info_log {
    ( $id:expr, $fn_iv:path, $fn_log:path ) => {
        {
            let log_len = unsafe {
                let mut log_len = 0;
                $fn_iv(
                    $id,
                    gl::INFO_LOG_LENGTH,
                    &mut log_len as *mut GLint,
                );
                log_len as usize
            };
            let mut array = vec![0; log_len];
            let mut actual_len = 0;
            unsafe {
                $fn_log(
                    $id,
                    array.len() as GLsizei,
                    &mut actual_len as *mut GLsizei,
                    array.as_mut_ptr() as *mut GLchar,
                );
            }
            array.truncate((actual_len + 1) as usize);
            CStr::from_bytes_with_nul(&array).unwrap().to_owned()
        }
    };
}

fn compile_shader(type_: GLenum, text: &[u8]) -> Result<GLuint, CString> {
    let lines = [text.as_ptr()];
    let lengths = [text.len()];
    let id = unsafe { gl::CreateShader(type_) };
    let success = unsafe {
        gl::ShaderSource(
            id,
            lines.len() as GLsizei,
            lines.as_ptr() as *const *const GLchar,
            lengths.as_ptr() as *const GLint,
        );
        gl::CompileShader(id);
        let mut success: GLint = 0;
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success as *mut GLint);
        success
    };
    if success != gl::FALSE as GLint {
        Ok(id)
    } else {
        let log = info_log!(id, gl::GetShaderiv, gl::GetShaderInfoLog);
        unsafe { gl::DeleteShader(id) };
        Err(log)
    }
}

const VERTEX_SOURCE: &'static [u8] = include_bytes!(
    "standard_vertex.glsl");
const FRAGMENT_SOURCE: &'static [u8] = include_bytes!(
    "standard_fragment.glsl");
const TEXT_VERTEX_SOURCE: &'static [u8] = include_bytes!(
    "text_vertex.glsl");
const TEXT_FRAGMENT_SOURCE: &'static [u8] = include_bytes!(
    "text_fragment.glsl");

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
        unsafe { gl::DeleteProgram(self.id); }
    }
}

fn compile_program(vert_text: &[u8], frag_text: &[u8])
    -> Result<ProgramObject, ProgramCompileError>
{
    let vert_id = compile_shader(gl::VERTEX_SHADER, vert_text)
        .map_err(ProgramCompileError::Vertex)?;
    let frag_id = compile_shader(gl::FRAGMENT_SHADER, frag_text)
        .map_err(ProgramCompileError::Fragment)?;
    let program;
    let success = unsafe {
        program = ProgramObject {
            id: gl::CreateProgram(),
        };
        gl::AttachShader(program.id, vert_id);
        gl::AttachShader(program.id, frag_id);
        gl::LinkProgram(program.id);
        let mut success: GLint = 0;
        gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut success as *mut GLint);
        success != gl::FALSE as GLint
    };
    let err = if success {
        Ok(())
    } else {
        Err(ProgramCompileError::Link(
            info_log!(program.id, gl::GetProgramiv, gl::GetProgramInfoLog)
        ))
    };
    unsafe {
        gl::DetachShader(program.id, vert_id);
        gl::DetachShader(program.id, frag_id);
        gl::DeleteShader(vert_id);
        gl::DeleteShader(frag_id);
    }
    err.map(|_| program)
}

#[derive(Clone)]
pub struct Shader {
    _obj: Rc<ProgramObject>,
    program_id: GLuint,
}

#[derive(Copy, Clone, Debug)]
pub struct UniformLoc {
    id: GLint,
}

impl Shader {
    pub fn create(vert_text: &[u8], frag_text: &[u8])
        -> Result<Self, ProgramCompileError>
    {
        let obj = compile_program(vert_text, frag_text)?;
        let shader = Shader {
            program_id: obj.id,
            _obj: Rc::new(obj),
        };
        Ok(shader)
    }

    pub fn standard() -> Self {
        Self::create(VERTEX_SOURCE, FRAGMENT_SOURCE).expect(
            "Standard Shader failed to compile"
        )
    }

    pub fn text() -> Self {
        Self::create(TEXT_VERTEX_SOURCE, TEXT_FRAGMENT_SOURCE).expect(
            "Text Shader failed to compile"
        )
    }

    pub fn make_current(&self) {
        unsafe { gl::UseProgram(self.program_id) };
    }

    pub fn uniform(&mut self, name: &str) -> UniformLoc {
        let prog_id = self.program_id;
        let cname = CString::new(name)
            .expect("Uniform name contained null");
        let id = unsafe {
            gl::GetUniformLocation(
                prog_id,
                cname.as_ptr() as *const GLchar,
            )
        };
        debug_assert_ne!(id, -1, "Failed to get uniform {}", name);
        UniformLoc { id }
    }

    pub fn set_opaque(&mut self, loc: UniformLoc, value: GLuint) {
        unsafe {
            gl::Uniform1i(loc.id, value as GLint);
        }
    }

    pub fn set_float(&mut self, loc: UniformLoc, value: GLfloat) {
        unsafe {
            gl::Uniform1f(loc.id, value);
        }
    }

    pub fn set_vec2(&mut self, loc: UniformLoc, value: (GLfloat, GLfloat)) {
        unsafe {
            gl::Uniform2f(loc.id, value.0, value.1);
        }
    }

    pub fn set_vec4(
        &mut self,
        loc: UniformLoc,
        value: (GLfloat, GLfloat, GLfloat, GLfloat),
    ) {
        unsafe {
            gl::Uniform4f(
                loc.id,
                value.0, value.1, value.2, value.3,
            );
        }
    }
}
