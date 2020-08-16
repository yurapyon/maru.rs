use gl::{
    self,
    types::*,
};

use super::{
    Program,
    Uniform,
};

//

// note: sometimes location will be -1
//       if location can not be found
//       just ignore it gracefully
//         or have separate 'new' function that reports error
#[derive(Debug)]
pub struct Location {
    location: GLint
}

impl Location {
    pub fn new(program: &Program, name: &str) -> Self {
        use std::ffi::CString;

        let c_str = CString::new(name.as_bytes()).unwrap();
        unsafe {
            let location = gl::GetUniformLocation(program.gl(), c_str.as_ptr() as _);
            Self {
                location,
            }
        }
    }

    pub fn location(&self) -> GLint {
        self.location
    }

    pub fn set<T: Uniform>(&self, val: &T) {
        val.uniform(self);
    }
}
