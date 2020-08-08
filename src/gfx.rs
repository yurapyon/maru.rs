//! # Graphics
//!
//! graphics module

// #![allow(dead_code)]

use std::{
    ffi::CString,
    ptr,
    marker::PhantomData,
    mem,
};

use gl::{
    self,
    types::*,
};
use image::{
    self,
    RgbaImage,
    Rgba
};
use nalgebra_glm as glm;

use crate::math::{
    AABB,
    Color,
};

//

// note: GL types are only used for things that interface directly with ogl
//         such as object handles and enums
//       otherwise just use native types based off of:
//         https://www.khronos.org/opengl/wiki/OpenGL_Type

//

// TODO
//   think about errors on new
//     only thing you really need to check errors on is shader compile and program link
//     other stuff is mostly data errors, out of bounds errors, etc
//       dont bother? makes for dirty api

// some type of GL trait?
//   unsafe from GLuint
//   to GLuint

// do fns that dont modify rust data but modify GLdata have to be &mut self?
//    bind and unbind dont need to be mutable
//    anything that does change gl properties of an object yes
//      anything that changes data
//      bind and unbind allow user to change data of bound obj and should technically be mut

// clone glfw context into every gl item,
//   dont have the problem with needing to put things in a certain order in structs
//   incerements glfw ref count
//   might not be guaranteed behavior by glfw lib,
//   but i could still do it on my own w/ an Rc<>
// only create objects through glfw context to manage lifetimes

// stuff for modifying underlying vec and making changes to the buf
//   maybe not super necessary if mapping buffers
/*
pub struct Backed<T: GLBuffer<O>> {
    buf: T,
    vec: Vec<O>,
}

grab a slice [], wrapped in something
on drop, will buffer the data to the buffer, just based on how much of the slice you got
*/

//

// TODO differentiate betw shader and program err?
#[derive(Debug)]
pub enum GfxError {
     BadInit(String),
}

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
    pub fn new(ty: GLenum, strings: &[&str]) -> Result<Self, GfxError> {
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
                return Err(GfxError::BadInit(String::from_utf8(buf).unwrap()))
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

//

/// Simple wrapper around an OpenGL program.
#[derive(Debug)]
pub struct Program {
    program: GLuint,
}

impl Program {
    /// Creates a new program from shaders.
    pub fn new(shaders: &[Shader]) -> Result<Self, GfxError> {
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
                return Err(GfxError::BadInit(String::from_utf8(buf).unwrap()))
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
        unsafe {
            let c_str = CString::new(name.as_bytes()).unwrap();
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

//

pub trait Uniform {
    fn uniform(&self, loc: &Location);
}

impl Uniform for f32 {
    fn uniform(&self, loc: &Location) {
        unsafe {
            gl::Uniform1f(loc.location(), *self);
        }
    }
}

impl Uniform for bool {
    fn uniform(&self, loc: &Location) {
        unsafe {
            gl::Uniform1i(loc.location(), if *self { 1 } else { 0 });
        }
    }
}

impl Uniform for i32 {
    fn uniform(&self, loc: &Location) {
        unsafe {
            gl::Uniform1i(loc.location(), *self);
        }
    }
}

impl Uniform for u32 {
    fn uniform(&self, loc: &Location) {
        unsafe {
            gl::Uniform1ui(loc.location(), *self);
        }
    }
}

impl Uniform for glm::Vec4 {
    fn uniform(&self, loc: &Location) {
        unsafe {
            let buf: &[f32; 4] = self.as_ref();
            gl::Uniform4fv(loc.location(), 1, buf.as_ptr());
        }
    }
}

impl Uniform for Color {
    fn uniform(&self, loc: &Location) {
        unsafe {
            let buf: &[GLfloat; 4] = self.as_ref();
            gl::Uniform4fv(loc.location(), 1, buf.as_ptr());
        }
    }
}

impl Uniform for glm::Mat3 {
    fn uniform(&self, loc: &Location) {
        unsafe {
            let buf: &[f32] = self.as_slice();
            gl::UniformMatrix3fv(loc.location(), 1, gl::FALSE, buf.as_ptr());
        }
    }
}

#[derive(Debug)]
pub struct TextureData<'a> {
    pub select: GLenum,
    pub bind_to: GLenum,
    pub texture: &'a Texture,
}

impl<'a> TextureData<'a> {
    // TODO get rid of these
    pub fn diffuse(texture: &'a Texture) -> Self {
        Self {
            select: gl::TEXTURE0,
            bind_to: gl::TEXTURE_2D,
            texture,
        }
    }

    pub fn normal(texture: &'a Texture) -> Self {
        Self {
            select: gl::TEXTURE1,
            bind_to: gl::TEXTURE_2D,
            texture,
        }
    }
}

impl Uniform for TextureData<'_> {
    fn uniform(&self, loc: &Location) {
        (self.select - gl::TEXTURE0).uniform(loc);
        unsafe {
            gl::ActiveTexture(self.select);
            gl::BindTexture(self.bind_to, self.texture.gl());
        }
    }
}

//

// TODO
//   buffer data
/// Simple wrapper around an OpenGL texture.
#[derive(Debug)]
pub struct Texture {
    texture: GLuint,
    width: u32,
    height: u32,
}

impl Texture {
    /// Create a new texture from an RgbaImage.
    pub fn new(image: &RgbaImage) -> Self {
        unsafe {
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            // TODO
            //   endianness not neccessary cuz Rgba<u8> is [u8; 4]
            //   why do i have to do reverse though?
            /*
            let ty = if cfg!(target_endian = "little") {
                    gl::UNSIGNED_INT_8_8_8_8
                } else {
                    gl::UNSIGNED_INT_8_8_8_8_REV
                };
            */

            let ty = gl::UNSIGNED_INT_8_8_8_8_REV;

            let i_vec = image.clone().into_vec();

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint,
                image.width() as GLint, image.height() as GLint, 0,
                gl::RGBA, ty, i_vec.as_ptr() as _);

            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            let mut ret = Self {
                texture,
                width: image.width(),
                height: image.height(),
            };

            ret.set_wrap(gl::REPEAT, gl::REPEAT);
            ret.set_filter(gl::LINEAR, gl::LINEAR);
            ret
        }
    }

    /// Set wrap mode for texture.
    pub fn set_wrap(&mut self, s: GLenum, t: GLenum) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, s as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, t as GLint);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    /// Set filter mode for texture.
    pub fn set_filter(&mut self, min: GLenum, mag: GLenum) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag as GLint);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    /// Set border color for texture.
    // TODO take maru::Color?
    pub fn set_border_color(&mut self, r: GLfloat, g: GLfloat, b: GLfloat, a: GLfloat) {
        unsafe {
            let tmp = [r, g, b, a];
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, tmp.as_ptr());
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    fn _get_dimensions(texture: GLuint, level: GLint) -> (u32, u32) {
        unsafe {
            let mut width = 0;
            let mut height = 0;
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::GetTexLevelParameteriv(gl::TEXTURE_2D, level, gl::TEXTURE_WIDTH, &mut width);
            gl::GetTexLevelParameteriv(gl::TEXTURE_2D, level, gl::TEXTURE_HEIGHT, &mut height);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            (width as u32, height as u32)
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn gl(&self) -> GLuint {
        self.texture
    }
}

impl From<GLuint> for Texture {
    fn from(texture: GLuint) -> Self {
        let (width, height) = Self::_get_dimensions(texture, 1);
        Self {
            texture,
            width,
            height,
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &mut self.texture);
        }
    }
}

pub type TextureRegion = AABB<u32>;
pub type UvRegion = AABB<f32>;

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

pub trait Vertex {
    /// A `Buffer<Self>` will be bound to `gl::ARRAY_BUFFER`.
    /// Use this function to set VertexAttributes in the vao.
    fn set_attributes(vao: &mut VertexArray);
}

//

// TODO allow user access to mesh?
/// Optimized instancer of a mesh.
pub struct Instancer<T: Vertex> {
    ibo: Buffer<T>,
    vec: Vec<T>,
}

impl<T: Vertex> Instancer<T> {
    pub fn new(size: usize) -> Self {
        let ibo = Buffer::empty(size, gl::STREAM_DRAW);
        let vec = Vec::with_capacity(size);

        Self {
            ibo,
            vec,
        }
    }

    pub fn make_mesh_compatible<M: Vertex>(&self, mesh: &mut Mesh<M>) {
        let vao = mesh.vao_mut();

        vao.bind();
        self.ibo.bind_to(gl::ARRAY_BUFFER);
        T::set_attributes(vao);
        vao.unbind();
    }

    /// note: does not change OpenGL state to bind
    pub fn bind<'a, M: Vertex>(&'a mut self, mesh: &'a Mesh<M>) -> BoundInstancer<'a, T, M> {
        let mut ret = BoundInstancer {
            base: self,
            mesh,
        };
        ret.begin();
        ret
    }

    pub fn fill_count(&self) -> usize {
        self.vec.len()
    }

    pub fn empty_count(&self) -> usize {
        self.vec.capacity() - self.vec.len()
    }
}

pub struct BoundInstancer<'a, T: Vertex, M: Vertex> {
    base: &'a mut Instancer<T>,
    mesh: &'a Mesh<M>,
}

impl<'a, T: Vertex, M: Vertex> BoundInstancer<'a, T, M> {
    /// note: does not change OpenGL state
    fn begin(&mut self) {
        self.base.vec.clear();
    }

    fn end(&mut self) {
        if self.base.fill_count() > 0 {
            self.draw();
        }
    }

    /// note: not expensive to call if instancer is empty
    pub fn draw(&mut self) {
        if self.base.fill_count() > 0 {
            self.base.ibo.buffer_sub_data(0, &self.base.vec);
            self.mesh.draw_instanced(self.base.fill_count());
            self.clear();
        }
    }

    pub fn clear(&mut self) {
        self.base.vec.clear();
    }

    pub fn push(&mut self, obj: T) {
        if self.base.empty_count() == 0 {
            self.draw();
        }
        self.base.vec.push(obj);
    }

    /// Returns an &mut T for the caller to override.
    /// Will be uninitialized.
    /// note: draw call may happen such that this &mut T will not be included in it
    pub fn pull(&mut self) -> &mut T {
        if self.base.empty_count() == 0 {
            self.draw();
        }
        let len = self.base.vec.len();
        let ret = unsafe {
            self.base.vec.set_len(len + 1);
            self.base.vec.get_unchecked_mut(len)
        };
        ret
    }
}

impl<'a, T: Vertex + Default, M: Vertex> BoundInstancer<'a, T, M> {
    /// Returns an &mut T for the caller to override.
    /// Will be initialized with `DEfault::default()`.
    /// note: draw call may happen such that this &mut T will not be included in it
    pub fn pull_default(&mut self) -> &mut T {
        let ret = self.pull();
        *ret = Default::default();
        ret
    }
}

impl<'a, T: Vertex, M: Vertex> Drop for BoundInstancer<'a, T, M> {
    fn drop(&mut self) {
        self.end();
    }
}

//

pub struct Canvas {
    texture: Texture,
    fbo: GLuint,
    rbo: GLuint,
}

// TODO structs for renderbuffer and framebuffer
impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        let img = RgbaImage::from_pixel(width, height, Rgba::from([0, 0, 0, 255]));
        let texture = Texture::new(&img);

        unsafe {
            let mut rbo = 0;
            let mut fbo = 0;

            gl::GenRenderbuffers(1, &mut rbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
            gl::RenderbufferStorage(gl::RENDERBUFFER,
                gl::DEPTH24_STENCIL8,
                width as GLsizei,
                height as GLsizei);
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture.gl(), 0);
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                rbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            Self {
                texture,
                fbo,
                rbo,
            }
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn set_gl_viewport(&self) {
        unsafe {
            gl::Viewport(0, 0, self.texture.width() as GLsizei, self.texture.height() as GLsizei);
        }
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &mut self.fbo);
            gl::DeleteRenderbuffers(1, &mut self.rbo);
        }
    }
}

//

pub struct Mesh<T: Vertex> {
    vao: VertexArray,
    vbo: Buffer<T>,
    vertices: Vec<T>,
    ebo: Buffer<u32>,
    indices: Vec<u32>,
    _buffer_type: GLenum,
    draw_type: GLenum,
}

impl<T: Vertex> Mesh<T> {
    pub fn new(vertices: Vec<T>, indices: Vec<u32>, buffer_type: GLenum, draw_type: GLenum) -> Self {
        let mut vao = VertexArray::new();
        let vbo = Buffer::from_slice(&vertices, buffer_type);
        let ebo = Buffer::from_slice(&indices, buffer_type);

        vao.bind();
        vbo.bind_to(gl::ARRAY_BUFFER);
        T::set_attributes(&mut vao);
        ebo.bind_to(gl::ELEMENT_ARRAY_BUFFER);
        vao.unbind();

        Self {
            vao,
            vbo,
            vertices,
            ebo,
            indices,
            _buffer_type: buffer_type,
            draw_type,
        }
    }

    pub fn draw(&self) {
        let i_ct = self.indices.len();

        self.vao.bind();
        if i_ct == 0 {
            unsafe {
                gl::DrawArrays(self.draw_type,
                               0,
                               self.vertices.len() as GLint);
            }
        } else {
            unsafe {
                gl::DrawElements(self.draw_type,
                                 i_ct as GLint,
                                 gl::UNSIGNED_INT,
                                 ptr::null());
            }
        }
        self.vao.unbind();
    }

    pub fn draw_instanced(&self, n: usize) {
        let i_ct = self.indices.len();

        self.vao.bind();
        if i_ct == 0 {
            unsafe {
                gl::DrawArraysInstanced(self.draw_type,
                                        0,
                                        self.vertices.len() as GLint,
                                        n as GLint);
            }
        } else {
            unsafe {
                gl::DrawElementsInstanced(self.draw_type,
                                          i_ct as GLint,
                                          gl::UNSIGNED_INT,
                                          ptr::null(),
                                          n as GLint);
            }
        }
        self.vao.unbind();
    }

    pub fn buffer_data(&mut self) {
        self.vbo.buffer_data(&self.vertices);
        self.ebo.buffer_data(&self.indices);
    }

    // TODO
    // pub fn vertices_mut(&mut self) -> &mut Vertices {
        // &mut self.vertices
    // }

    /// Just be careful
    pub fn vao_mut(&mut self) -> &mut VertexArray {
        &mut self.vao
    }
}
