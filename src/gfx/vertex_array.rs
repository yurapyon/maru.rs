use gl::{
    self,
    types::*,
};

//

/// OpenGL Vertex Attribute type.
#[derive(Copy, Clone)]
pub struct VertexAttribute {
    pub size: GLint,
    pub ty: GLenum,
    pub normalized: bool,
    pub stride: usize,
    pub offset: usize,
    pub divisor: GLuint,
}

//

pub trait Vertex {
    /// A `Buffer<Self>` will be bound to `gl::ARRAY_BUFFER`.
    /// Use this function to set VertexAttributes in the vao.
    fn set_attributes(vao: &mut VertexArray);
}

//

/// Simple wrapper around an OpenGL vertex array.
pub struct VertexArray {
    vao: GLuint,
}

impl VertexArray {
    /// Creates a new vertex array.
    pub fn new() -> Self {
        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }
        Self {
            vao
        }
    }

    /// Adds and enables a vertex attribute by number.
    pub fn enable_attribute(&mut self, num: GLuint, attrib: VertexAttribute) {
        unsafe {
            // note: redundant
            gl::BindVertexArray(self.vao);
            gl::EnableVertexAttribArray(num);
            gl::VertexAttribPointer(
                num,
                attrib.size,
                attrib.ty,
                if attrib.normalized { gl::TRUE } else { gl::FALSE },
                attrib.stride as GLsizei,
                attrib.offset as _);
            gl::VertexAttribDivisor(num, attrib.divisor);
        }
    }

    /// Disables a vertex attribute by number.
    pub fn disable_attribute(&mut self, num: GLuint) {
        unsafe {
            // note: redundant
            gl::BindVertexArray(self.vao);
            gl::DisableVertexAttribArray(num);
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }

    pub fn gl(&self) -> GLuint {
        self.vao
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &mut self.vao);
        }
    }
}
