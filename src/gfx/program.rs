use gl::{
    self,
    types::*,
};

use super::Shader;

//

/// Simple wrapper around an OpenGL program.
#[derive(Debug)]
pub struct Program {
    program: GLuint,
}

impl Program {
    /// Creates a new program from shaders.
    pub fn new(shaders: &[Shader]) -> Result<Self, String> {
        use std::ptr;

        unsafe {
            let program = gl::CreateProgram();
            for shd in shaders {
                gl::AttachShader(program, shd.gl());
            }
            gl::LinkProgram(program);

            let mut success = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);

            if success != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);

                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1);

                gl::GetProgramInfoLog(program,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar);
                gl::DeleteProgram(program);
                return Err(String::from_utf8(buf).unwrap())
            }

            Ok(Self { program })
        }
    }

    /// `gl::UseProgram();`
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }

    pub fn gl(&self) -> GLuint {
        self.program
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}
