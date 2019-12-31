use std::collections::HashMap;
use std::ffi::{CStr, CString};

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
            array.truncate(actual_len as usize);
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

pub enum ProgramCompileError {
    Vertex(CString),
    Fragment(CString),
    Link(CString),
}

fn compile_program(vert_text: &[u8], frag_text: &[u8])
    -> Result<GLuint, ProgramCompileError>
{
    let vert_id = compile_shader(gl::VERTEX_SHADER, vert_text)
        .map_err(ProgramCompileError::Vertex)?;
    let frag_id = compile_shader(gl::FRAGMENT_SHADER, frag_text)
        .map_err(ProgramCompileError::Fragment)?;
    let prog_id;
    let success = unsafe {
        prog_id = gl::CreateProgram();
        gl::AttachShader(prog_id, vert_id);
        gl::AttachShader(prog_id, frag_id);
        gl::LinkProgram(prog_id);
        let mut success: GLint = 0;
        gl::GetProgramiv(prog_id, gl::LINK_STATUS, &mut success as *mut GLint);
        success
    };
    let result = if success != gl::FALSE as GLint {
        Ok(prog_id)
    } else {
        Err(ProgramCompileError::Link(
            info_log!(prog_id, gl::GetProgramiv, gl::GetProgramInfoLog)
        ))
    };
    unsafe {
        gl::DetachShader(prog_id, vert_id);
        gl::DetachShader(prog_id, frag_id);
        gl::DeleteShader(vert_id);
        gl::DeleteShader(frag_id);
    }
    if result.is_err() {
        unsafe { gl::DeleteProgram(prog_id) };
    }
    result
}

pub struct Shader {
    program_id: GLuint,
    uniforms: HashMap<String, GLint>,
}

impl Shader {
    pub fn create(vert_text: &[u8], frag_text: &[u8])
        -> Result<Self, ProgramCompileError>
    {
        let shader = Shader {
            program_id: compile_program(vert_text, frag_text)?,
            uniforms: HashMap::new(),
        };
        Ok(shader)
    }

    pub fn standard() -> Result<Self, ProgramCompileError> {
        Self::create(VERTEX_SOURCE, FRAGMENT_SOURCE)
    }

    pub fn set_current(&self) {
        unsafe { gl::UseProgram(self.program_id) };
    }

    pub fn set_uniform_float(&mut self, name: &str, value: GLfloat) {
        let prog_id = self.program_id;
        let uid = *self.uniforms.entry(name.into()).or_insert_with(|| {
            let name = CString::new(name)
                .expect("Uniform name contained null");
            unsafe {
                gl::GetUniformLocation(
                    prog_id,
                    name.as_ptr() as *const GLchar,
                )
            }
        });
        assert_ne!(uid, -1);
        unsafe {
            gl::Uniform1f(uid, value);
        }
    }
}
