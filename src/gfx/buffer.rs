use std::{
    marker::PhantomData,
    mem,
};

use gl::{
    self,
    types::*,
};

//

/// Simple wrapper around an OpenGL buffer.
pub struct Buffer<T> {
    buffer: GLuint,
    usage_type: GLenum,
    _phantom: PhantomData<T>,
}

impl<T> Buffer<T> {
    /// Create a new buffer, will be uninitialized.
    /// Prefer using `Buffer::empty()` or `Buffer::from_slice()`.
    unsafe fn new(usage_type: GLenum) -> Self {
        let mut buffer = 0;
        #[allow(unused_unsafe)]
        unsafe {
            gl::GenBuffers(1, &mut buffer);
        }
        Self {
            buffer,
            usage_type,
            _phantom: PhantomData,
        }
    }

    /// Creates a new buffer of size `len`.
    pub fn empty(len: usize, usage_type: GLenum) -> Self {
        let mut ret = unsafe {
            Self::new(usage_type)
        };
        ret.buffer_null(len);
        ret
    }

    /// Creates a new buffer from a slice.
    pub fn from_slice(slice: &[T], usage_type: GLenum) -> Self {
        let mut ret = unsafe {
            Self::new(usage_type)
        };
        ret.buffer_data(slice);
        ret
    }

    /// Reinitializes buffer to size `len`.
    pub fn buffer_null(&mut self, len: usize) {
        use std::ptr;

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer);
            gl::BufferData(gl::ARRAY_BUFFER,
                (len * mem::size_of::<T>()) as GLsizeiptr,
                ptr::null(),
                self.usage_type);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    /// Reinitializes buffer from a slice.
    // TODO have this be mut?
    //      buffer sub data isnt mut
    pub fn buffer_data(&mut self, data: &[T]) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer);
            gl::BufferData(gl::ARRAY_BUFFER,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as _,
                self.usage_type);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    /// Subs data into buffer from a slice.
    // TODO check buffer size before doing this
    //        have to keep in struct
    //      check what opengl does if writing something too big
    pub fn buffer_sub_data(&self, offset: usize, data: &[T]) {
        if data.len() == 0 {
            return;
        }
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer);
            gl::BufferSubData(gl::ARRAY_BUFFER,
                offset as GLintptr,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as _);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    // TODO map buffer
    //   could use some other type that automatically unmaps when leaving scope
    //   like mutex lock

    pub fn bind_to(&self, target: GLenum) {
        unsafe {
            gl::BindBuffer(target, self.buffer);
        }
    }

    pub fn unbind_from(&self, target: GLenum) {
        unsafe {
            gl::BindBuffer(target, 0);
        }
    }

    pub fn gl(&self) -> GLuint {
        self.buffer
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.buffer);
        }
    }
}
