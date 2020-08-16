use gl::{
    self,
    types::*,
};

//

// TODO think about using Newtype
//    not going to be any different
//    only thing would be optimization i guess

/// Simple wrapper around an OpenGL shader.
#[derive(Debug)]
pub struct Shader {
    shader: GLuint,
}

impl Shader {
    // TODO report errors better
    //        also report warnings somehow?
    //      does it really need to take an array of strs?
    //        if you had an arry of strs you could just do [].concat() into a String.
    /// Creates a new shader from strings.
    pub fn new(ty: GLenum, strings: &[&str]) -> Result<Self, String> {
        use std::ffi::CString;
        use std::ptr;

        let c_strs: Vec<_> = strings.iter()
            .map(| s | CString::new(s.as_bytes()).unwrap())
            .collect();

        let c_ptrs: Vec<_> = c_strs.iter()
            .map(| s | s.as_ptr())
            .collect();

        unsafe {
            let shader = gl::CreateShader(ty);

            gl::ShaderSource(shader, c_ptrs.len() as GLsizei, c_ptrs.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            let mut success = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

            if success != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1);

                gl::GetShaderInfoLog(shader,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar);
                gl::DeleteShader(shader);
                return Err(String::from_utf8(buf).unwrap())
            }

            Ok(Self { shader })
        }
    }

    pub fn gl(&self) -> GLuint {
        self.shader
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.shader);
        }
    }
}
