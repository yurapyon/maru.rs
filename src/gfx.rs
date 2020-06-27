//! # Graphics
//!
//! graphics module

#![allow(dead_code)]

use std::{
    ffi::CString,
    ptr,
    marker::PhantomData,
    mem,
};

use cgmath::{
    prelude::*,
    Vector2,
    Vector4,
    Matrix4,
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
use memoffset::offset_of;

//

use crate::math::{
    ext::*,
    Transform2d,
    Vertex,
    Vertices,
    Color,
    AABB,
};

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
//   seems like they should be
//     links idea of rust objects to gl objects
//   open gl isnt clear abt what will mutate data
// prev:
//    bind and unbind dont need to be mutable?
//    anything that does change gl properties of an object yes

// maru premade 2dcontext
//   dont build it over and over for each game?
//   namespace it into default2d or something
// all defaults can be moved into here ?
//   not very good for the api
//   also the idea isnt to have reuasable open gl utils
//     just so that you can kindof bend the library to use the utils on thier own
//     but the main idea is a higher level library
//       pretty thin wrapper around openGl,

// prelude with reexports

//

// TODO use string?
//      can just use &str maybe
//        then have to worry abt lifetimes
// differentiate betw shader and program err?
#[derive(Debug)]
pub enum GfxError {
     BadInit(String),
}

//

/// Holds information about maru's default shader format.
///
/// # Example
///
/// TODO: shader template format example
pub struct ShaderTemplate {
    header: String,
    extras: String,
    effect: String,
    footer: String,
}

impl ShaderTemplate {
    /// Creates a new shader template.
    pub fn new(base: &str, extras: Option<&str>) -> Result<Self, GfxError> {
        if !base.is_ascii() {
            return Err(GfxError::BadInit(String::from("base string must be ascii")));
        }

        let ct = base.chars()
            .filter(| &x | x == '@')
            .count();
        if ct != 2 {
            return Err(GfxError::BadInit(String::from("invalid base string")));
        }

        let mut strs = base.split('@');
        let header = String::from(strs.next().unwrap());
        let effect = String::from(strs.next().unwrap());
        let footer = String::from(strs.next().unwrap());

        let extras = match extras {
            Some(s) => String::from(s),
            None    => String::new(),
        };

        Ok(Self {
            header,
            extras,
            effect,
            footer,
        })
    }
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
    //      also report warnings somehow?
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

    /// Creates a new shader from a template, with optional replacement effect.
    pub fn from_template(ty: GLenum, st: &ShaderTemplate, effect: Option<&str>) -> Result<Self, GfxError> {
        let effect = match effect {
            Some(s) => s,
            None    => &st.effect,
        };

        let strs = [
            &st.header,
            &st.extras,
            effect,
            &st.footer,
        ];

        Shader::new(ty, &strs)
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

    // TODO dont allow return errors ?
    //     just panic
    /// Creates a default maru program, with optionial vert and frag effects.
    pub fn new_default(v_effect: Option<&str>, f_effect: Option<&str>) -> Result<Self, GfxError> {
        use crate::content;

        let vert = Shader::from_template(gl::VERTEX_SHADER,
            &ShaderTemplate::new(content::shaders::DEFAULT_VERT,
                Some(content::shaders::EXTRAS))?,
            v_effect,
        )?;

        let frag = Shader::from_template(gl::FRAGMENT_SHADER,
            &ShaderTemplate::new(content::shaders::DEFAULT_FRAG,
                Some(content::shaders::EXTRAS))?,
            f_effect,
        )?;

        Program::new(&[vert, frag])
    }

    // TODO dont allow return errors ?
    /// Creates a default maru spritebatch program.
    /// Effects cannot be suppiled as with `Program::new_default()`.
    pub fn new_default_spritebatch() -> Result<Self, GfxError> {
        use crate::content;

        let vert = Shader::from_template(gl::VERTEX_SHADER,
            &ShaderTemplate::new(content::shaders::DEFAULT_VERT,
                Some(content::shaders::EXTRAS))?,
            Some(content::shaders::DEFAULT_SB_VERT),
        )?;

        let frag = Shader::from_template(gl::FRAGMENT_SHADER,
            &ShaderTemplate::new(content::shaders::DEFAULT_FRAG,
                Some(content::shaders::EXTRAS))?,
            Some(content::shaders::DEFAULT_SB_FRAG),
        )?;

        Program::new(&[vert, frag])
    }

    /// `gl::UseProgram();`
    pub fn gl_use(&self) {
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

// TODO
//   buffer data
//   switch width and height to usizes?
/// Simple wrapper around an OpenGL texture.
pub struct Texture {
    texture: GLuint,
    width: GLint,
    height: GLint,
}

impl Texture {
    /// Create a new texture from an RgbaImage.
    pub fn new(image: &RgbaImage) -> Self {
        unsafe {
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            // TODO
            //   endianness not neccessary cuz Rgba<u8> is fixed order
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
                width: image.width() as GLint,
                height: image.height() as GLint,
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

    fn _get_dimensions(texture: GLuint, level: GLint) -> (GLint, GLint) {
        unsafe {
            let mut width = 0;
            let mut height = 0;
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::GetTexLevelParameteriv(gl::TEXTURE_2D, level, gl::TEXTURE_WIDTH, &mut width);
            gl::GetTexLevelParameteriv(gl::TEXTURE_2D, level, gl::TEXTURE_HEIGHT, &mut height);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            (width, height)
        }
    }

    pub fn width(&self) -> GLint {
        self.width
    }

    pub fn height(&self) -> GLint {
        self.height
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
    pub fn buffer_sub_data(&self, offset: usize, data: &[T]) {
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

//

/// Useful for instancing and batching.
pub struct InstanceBuffer<T> {
    ibo: Buffer<T>,
    buffer: Vec<T>,
    index: usize,
}

impl<T> InstanceBuffer<T> {
    pub fn new(len: usize) -> Self {
        let ibo = Buffer::empty(len, gl::STREAM_DRAW);
        let mut buffer = Vec::with_capacity(len);
        unsafe {
            buffer.set_len(len);
        }

        Self {
            ibo,
            buffer,
            index: 0,
        }
    }

    pub fn clear(&mut self) {
        self.index = 0;
    }

    pub fn push(&mut self, obj: T) {
        if self.empty_count() == 0 {
            return;
        }
        self.buffer[self.index] = obj;
        self.index += 1;
    }

    pub fn pull(&mut self) -> Option<&mut T> {
        if self.index >= self.buffer.len() {
            None
        } else {
            unsafe {
                let ret = self.buffer.get_unchecked_mut(self.index);
                self.index += 1;
                Some(ret)
            }
        }
    }

    pub fn buffer_data(&mut self) {
        if self.index == 0 {
            return;
        }

        self.ibo.buffer_sub_data(0, &self.buffer[0..self.index]);
    }

    pub fn fill_count(&self) -> usize {
        self.index
    }

    pub fn empty_count(&self) -> usize {
        self.buffer.len() - self.index
    }

    pub(in crate::gfx) fn ibo(&self) -> &Buffer<T> {
        &self.ibo
    }
}

//

pub struct Canvas {
    texture: Texture,
    fbo: GLuint,
    rbo: GLuint,
}

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

pub struct Mesh {
    vertices: Vertices,
    vao: VertexArray,
    vbo: Buffer<Vertex>,
    ebo: Buffer<GLuint>,
    buffer_type: GLenum,
    draw_type: GLenum,
}

impl Mesh {
    pub fn new(vertices: Vertices, buffer_type: GLenum, draw_type: GLenum) -> Self {
        let mut vao = VertexArray::new();
        let vbo = Buffer::from_slice(&vertices.vertices, buffer_type);
        let ebo = Buffer::from_slice(&vertices.indices, buffer_type);

        let base = VertexAttribute {
            size: 3,
            ty: gl::FLOAT,
            normalized: false,
            stride: mem::size_of::<Vertex>(),
            offset: offset_of!(Vertex, position),
            divisor: 0,
        };

        vao.bind();
        vbo.bind_to(gl::ARRAY_BUFFER);
        vao.enable_attribute(0, VertexAttribute {
            offset: offset_of!(Vertex, position),
            .. base
        });
        vao.enable_attribute(1, VertexAttribute {
            offset: offset_of!(Vertex, normal),
            .. base
        });
        vao.enable_attribute(2, VertexAttribute {
            size: 2,
            offset: offset_of!(Vertex, uv),
            .. base
        });
        ebo.bind_to(gl::ELEMENT_ARRAY_BUFFER);
        vao.unbind();

        Self {
            vertices,
            vao,
            vbo,
            ebo,
            buffer_type,
            draw_type,
        }
    }

    pub fn draw(&self) {
        let i_ct = self.vertices.indices.len();

        self.vao.bind();
        if i_ct == 0 {
            unsafe {
                gl::DrawArrays(self.draw_type,
                    0,
                    self.vertices.vertices.len() as GLint);
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
        let i_ct = self.vertices.indices.len();

        self.vao.bind();
        if i_ct == 0 {
            unsafe {
                gl::DrawArraysInstanced(self.draw_type,
                    0,
                    self.vertices.vertices.len() as GLint,
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
        self.vbo.buffer_data(&self.vertices.vertices);
        self.ebo.buffer_data(&self.vertices.indices);
    }

    pub fn vertices_mut(&mut self) -> &mut Vertices {
        &mut self.vertices
    }

    /// Just be careful
    pub fn vao_mut(&mut self) -> &mut VertexArray {
        &mut self.vao
    }
}

//

// note: sometimes location will be -1
//       if location can not be found
//       just ignore it gracefully
//         or have separate 'new' function that reports error
#[derive(Copy, Clone)]
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
}

//

pub trait Uniform {
    fn uniform(&self, loc: Location);
}

impl Uniform for GLfloat {
    fn uniform(&self, loc: Location) {
        unsafe {
            gl::Uniform1f(loc.location(), *self);
        }
    }
}

impl Uniform for bool {
    fn uniform(&self, loc: Location) {
        unsafe {
            gl::Uniform1i(loc.location(), if *self { 1 } else { 0 });
        }
    }
}

impl Uniform for GLint {
    fn uniform(&self, loc: Location) {
        unsafe {
            gl::Uniform1i(loc.location(), *self);
        }
    }
}

impl Uniform for GLuint {
    fn uniform(&self, loc: Location) {
        unsafe {
            gl::Uniform1ui(loc.location(), *self);
        }
    }
}

impl Uniform for Vector4<GLfloat> {
    fn uniform(&self, loc: Location) {
        unsafe {
            let buf: &[GLfloat; 4] = self.as_ref();
            gl::Uniform4fv(loc.location(), 1, buf.as_ptr());
        }
    }
}

impl Uniform for Color {
    fn uniform(&self, loc: Location) {
        unsafe {
            let buf: &[GLfloat; 4] = self.as_ref();
            gl::Uniform4fv(loc.location(), 1, buf.as_ptr());
        }
    }
}

impl Uniform for Matrix4<GLfloat> {
    fn uniform(&self, loc: Location) {
        unsafe {
            let buf: &[GLfloat; 16] = self.as_ref();
            gl::UniformMatrix4fv(loc.location(), 1, gl::FALSE, buf.as_ptr());
        }
    }
}

pub struct TextureData<'a> {
    pub select: GLenum,
    pub bind_to: GLenum,
    pub texture: &'a Texture,
}

impl<'a> TextureData<'a> {
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
    fn uniform(&self, loc: Location) {
        (self.select - gl::TEXTURE0).uniform(loc);
        unsafe {
            gl::ActiveTexture(self.select);
            gl::BindTexture(self.bind_to, self.texture.gl());
        }
    }
}

//

pub struct DefaultLocations {
    projection: Location,
    view: Location,
    model: Location,
    time: Location,
    flip_uvs: Location,
    base_color: Location,
    tx_diffuse: Location,
    tx_normal: Location,
}

impl DefaultLocations {
    pub fn new(program: &Program) -> Self {
        Self {
            projection: Location::new(program, "_projection"),
            view:       Location::new(program, "_view"),
            model:      Location::new(program, "_model"),
            time:       Location::new(program, "_time"),
            flip_uvs:   Location::new(program, "_flip_uvs"),
            base_color: Location::new(program, "_base_color"),
            tx_diffuse: Location::new(program, "_tx_diffuse"),
            tx_normal:  Location::new(program, "_tx_normal"),
        }
    }

    pub fn projection(&self) -> Location {
        self.projection
    }

    pub fn view(&self) -> Location {
        self.view
    }

    pub fn model(&self) -> Location {
        self.model
    }

    pub fn time(&self) -> Location {
        self.time
    }

    pub fn flip_uvs(&self) -> Location {
        self.flip_uvs
    }

    pub fn base_color(&self) -> Location {
        self.base_color
    }

    pub fn diffuse(&self) -> Location {
        self.tx_diffuse
    }

    pub fn normal(&self) -> Location {
        self.tx_normal
    }
}

//

type TextureRegion = AABB<u32>;
type UvRegion = AABB<GLfloat>;

// just use ascii for now
// maybe in the future use utf8
//   but the lookup table would have to be different
//   idk
//     some type of convert string to vec<&textureRegion>
// TODO
//   inlucde medium sized font and large fon
//     new_default_small
//     new_default_medium
//     new_default_large
//     just take dont size as an enum
//       new_default(FontSize::Small) etc
//   make font with default alphabet
pub struct BitmapFont {
    texture: Texture,
    regions: Vec<TextureRegion>,
    uv_regions: Vec<UvRegion>,
}

impl BitmapFont {
    pub fn new(image: &RgbaImage, alphabet: &str) -> Self {
        let i_height = image.height();
        let base_color = image.get_pixel(0, 0);

        let top_row = image.enumerate_rows().next().unwrap().1;
        let breaks =
            top_row.filter_map(| (x, _y, px) | {
                if px == base_color {
                    Some(x)
                } else {
                    None
                }
            });
        let breaks_cl = breaks.clone();
        let pairs =
            breaks.zip(breaks_cl.skip(1))
                  .map(| (x1, x2) | TextureRegion::new(x1 + 1, 0, x2, i_height))
                  .zip(alphabet.chars());

        let mut regions = Vec::with_capacity(256);
        // set to 0,0,1,1 so if char is not found, uv is base_color
        regions.resize(256, TextureRegion::new(0, 0, 1, 1));

        for (region, ch) in pairs {
            regions[ch as usize] = region;
        }

        let mut texture = Texture::new(image);
        texture.set_wrap(gl::CLAMP_TO_EDGE, gl::CLAMP_TO_EDGE);
        texture.set_filter(gl::NEAREST, gl::NEAREST);

        let uv_regions = regions.iter()
                                .map(| region | region.normalized(&texture))
                                .collect();

        Self {
            texture,
            regions,
            uv_regions,
        }
    }

    pub fn new_default() -> Self {
        use crate::content;

        let fn_img = image::load_from_memory(content::image::SMALL_FONT).unwrap().to_rgba();
        Self::new(&fn_img,
        " ABCDEFGHIJKLMNOPQRSTUVWXYZ\
          abcdefghijklmnopqrstuvwxyz\
          1234567890[](){}=+-/^$@#*~%_<>\"'?!|\\&`.,:;")
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn region(&self, ch: char) -> TextureRegion {
        self.regions[ch as usize]
    }

    pub fn uv_region(&self, ch: char) -> UvRegion {
        self.uv_regions[ch as usize]
    }
}

//

#[derive(Debug)]
#[repr(C)]
pub struct SbSprite {
    pub uv: UvRegion,
    pub transform: Transform2d,
    pub color: Color,
}

impl Default for SbSprite {
    fn default() -> Self {
        Self {
            uv: UvRegion::new(0., 0., 1., 1.),
            transform: Default::default(),
            color: Color::new_rgba(1., 1., 1., 1.),
        }
    }
}

// use centered quad, for particles
pub struct Spritebatch {
    buffer: InstanceBuffer<SbSprite>,
    mesh_quad: Mesh,
}

impl Spritebatch {
    pub fn new(size: usize) -> Self {
        assert!(size != 0);

        let mut mesh_quad =
            Mesh::new(Vertices::quad(false),
                      gl::STATIC_DRAW,
                      gl::TRIANGLE_STRIP);


        let buffer = InstanceBuffer::new(size);
        let ibo = buffer.ibo();

        let vao = mesh_quad.vao_mut();

        let base = VertexAttribute {
            size: 4,
            ty: gl::FLOAT,
            normalized: false,
            stride: std::mem::size_of::<SbSprite>(),
            offset: 0,
            divisor: 1,
        };

        // TODO use enum for all the attrib locs
        vao.bind();
        ibo.bind_to(gl::ARRAY_BUFFER);
        vao.enable_attribute(3, VertexAttribute {
            offset: offset_of!(SbSprite, uv),
            .. base
        });
        vao.enable_attribute(4, VertexAttribute {
            size: 2,
            offset: offset_of!(SbSprite, transform) +
                    offset_of!(Transform2d, position),
            .. base
        });
        vao.enable_attribute(5, VertexAttribute {
            size: 2,
            offset: offset_of!(SbSprite, transform) +
                    offset_of!(Transform2d, scale),
            .. base
        });
        vao.enable_attribute(6, VertexAttribute {
            size: 1,
            offset: offset_of!(SbSprite, transform) +
                    offset_of!(Transform2d, rotation),
            .. base
        });
        vao.enable_attribute(7, VertexAttribute {
            offset: offset_of!(SbSprite, color),
            .. base
        });
        vao.unbind();

        Self {
            buffer,
            mesh_quad,
        }
    }

    pub fn begin(&mut self) {
        self.buffer.clear();
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
        }
    }

    pub fn draw(&mut self) {
        self.buffer.buffer_data();
        self.mesh_quad.draw_instanced(self.buffer.fill_count());
        self.buffer.clear();
    }

    pub fn end(&mut self) {
        if self.buffer.fill_count() > 0 {
            self.draw();
        }
    }

    // TODO is this a good interface?
    //      its ok
    //          maybe just return the ref and have the caller set defaults as necessary ?
    //   ulitmately copies defaults first then  returns ref
    //     c version allows you to change data then copy
    //     pretty much same thing, unless you map the buffer and write to it directly
    pub fn pull(&mut self) -> &mut SbSprite {
        if self.buffer.empty_count() == 0 {
            self.draw();
        }
        let ret = self.buffer.pull().unwrap();
        *ret = Default::default();
        ret
    }

    pub fn print(&mut self, font: &BitmapFont, text: &str) {
        self.begin();

        let mut x = 0.;
        let font_h = font.texture().height() as GLfloat;
        for ch in text.chars() {
            let region_w = font.region(ch).width() as GLfloat;
            let sp = self.pull();
            sp.uv = font.uv_region(ch);
            sp.transform.position.x = x;
            sp.transform.scale.x = region_w;
            sp.transform.scale.y = font_h;
            x += region_w;
        }

        self.end();
    }
}

//

// TODO where to put something like a coordinate system
//      put it in the default shader?
//        use view matrix
//      2d coordinate system
//        last after projection
//      put it in drawer?
//        wont work for anything else

// this is just a bunch of functions that generate uniforms on the fly
//   and applies them to current program
// also has meshes and textures

// polygon
// line
// triangle

// lined rect

pub struct Drawer {
    line_thickness: GLfloat,

    mesh_quad: Mesh,
    mesh_circle: Mesh,
    tex_white: Texture,
    tex_blue: Texture,
}

impl Drawer {
    pub fn new(circle_resolution: usize) -> Self {
        let white = RgbaImage::from_pixel(1, 1, Rgba::from([255, 255, 255, 255]));
        let blue = RgbaImage::from_pixel(1, 1, Rgba::from([0, 0, 255, 0]));
        Self {
            mesh_quad: Mesh::new(Vertices::quad(false),
                                 gl::STATIC_DRAW,
                                 gl::TRIANGLE_STRIP),
            mesh_circle: Mesh::new(Vertices::circle(circle_resolution),
                                   gl::STATIC_DRAW,
                                   gl::TRIANGLE_FAN),
            tex_white: Texture::new(&white),
            tex_blue: Texture::new(&blue),
            line_thickness: 2.0,
        }
    }

    pub fn line_thickness(&mut self) -> &mut GLfloat {
        &mut self.line_thickness
    }

    pub fn reset_uniforms(&self, locations: &DefaultLocations) {
        let m4_iden = Matrix4::identity();
        m4_iden.uniform(locations.projection());
        m4_iden.uniform(locations.view());
        m4_iden.uniform(locations.model());
        Color::white().uniform(locations.base_color());
        false.uniform(locations.flip_uvs());
        (0. as GLfloat).uniform(locations.time());
        TextureData::diffuse(&self.tex_white).uniform(locations.diffuse());
        TextureData::normal(&self.tex_blue).uniform(locations.normal());
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn sprite_px(&self, locations: &DefaultLocations, texture: &Texture, transform: &Transform2d) {
        TextureData::diffuse(texture).uniform(locations.diffuse());
        Matrix4::from_transform2d(transform).uniform(locations.model());
        self.mesh_quad.draw();
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn sprite(&self, locations: &DefaultLocations, texture: &Texture, transform: &Transform2d) {
        let temp = Transform2d {
            scale: Vector2::new(transform.scale.x * texture.width() as GLfloat,
                                transform.scale.y * texture.height() as GLfloat),
            .. *transform
        };
        self.sprite_px(locations, texture, &temp);
    }

    /// Sets locations.diffuse() and locations.model()
    ///   `rect` should be in the form: [x1, y1, x2, y2]
    pub fn filled_rectangle(&self, locations: &DefaultLocations, rect: Vector4<GLfloat>) {
        let temp = Transform2d {
            position: Vector2::new(rect.x, rect.y),
            scale:    Vector2::new(rect.z - rect.x,
                                   rect.w - rect.y),
            rotation: 0.,
        };
        self.sprite_px(locations, &self.tex_white, &temp);
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn circle(&self, locations: &DefaultLocations, x: GLfloat, y: GLfloat, r: GLfloat) {
        let transform = Transform2d {
            position: Vector2::new(x, y),
            scale:    Vector2::new(r, r),
            rotation: 0.,
        };
        TextureData::diffuse(&self.tex_white).uniform(locations.diffuse());
        Matrix4::from_transform2d(&transform).uniform(locations.model());
        self.mesh_circle.draw();
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn horizontal_line(&self, locations: &DefaultLocations, y: GLfloat, x1: GLfloat, x2: GLfloat) {
        let y1 = y - self.line_thickness / 2.;
        let y2 = y + self.line_thickness / 2.;
        self.filled_rectangle(locations, Vector4::new(x1, y1, x2, y2));
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn vertical_line(&self, locations: &DefaultLocations, x: GLfloat, y1: GLfloat, y2: GLfloat) {
        let x1 = x - self.line_thickness / 2.;
        let x2 = x + self.line_thickness / 2.;
        self.filled_rectangle(locations, Vector4::new(x1, y1, x2, y2));
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn line(&self, locations: &DefaultLocations, x1: GLfloat, y1: GLfloat, x2: GLfloat, y2: GLfloat) {
        if x1 == x2 {
            self.vertical_line(locations, x1, y1, y2);
        } else if y1 == y2 {
            self.horizontal_line(locations, y1, x1, x2);
        } else {
            // TODO
        }
    }
}
