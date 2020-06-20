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
    Vector2,
    Vector4,
    Matrix4,
};
use gl;
use gl::types::*;
use image;
use image::{
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
};

//

// TODO
//   think about errors on new
//     only thing you really need to check errors on is shader compile and program link
//     other stuff is mostly data errors, out of bounds errors, etc
//       dont bother? makes for dirty api
//  where mutability
//    bind and unbind dont need to be mutable?
//    anything that does change gl properties of an object yes

// some type of GL trait?
//   unsafe from GLuint
//   to GLuint

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
        // TODO why isnt this unsafe
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
        let mut img = RgbaImage::new(width, height);
        for p in img.pixels_mut() {
            p[3] = 255;
        }

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
        unsafe {
            let i_ct = self.vertices.indices.len();

            self.vao.bind();
            if i_ct == 0 {
                gl::DrawArrays(self.draw_type, 0, self.vertices.vertices.len() as GLint);
            } else {
                gl::DrawElements(self.draw_type, i_ct as GLint, gl::UNSIGNED_INT, ptr::null());
            }
            self.vao.unbind();
        }
    }

    pub fn draw_instanced(&self, n: usize) {
        unsafe {
            let i_ct = self.vertices.indices.len();

            self.vao.bind();
            if i_ct == 0 {
                gl::DrawArraysInstanced(self.draw_type, 0, self.vertices.vertices.len() as GLint, n as GLint);
            } else {
                gl::DrawElementsInstanced(self.draw_type, i_ct as GLint, gl::UNSIGNED_INT, ptr::null(), n as GLint);
            }
            self.vao.unbind();
        }
    }

    // TODO buffer data

    pub(in crate::gfx) unsafe fn vao_mut(&mut self) -> &mut VertexArray {
        &mut self.vao
    }
}

//

// note: sometimes location will be -1
//       if location can not be found
//       just ignore it gracefully
//         or have separate 'new' function that reports error
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
    fn uniform(&self, loc: &Location);
}

impl Uniform for GLfloat {
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

impl Uniform for GLint {
    fn uniform(&self, loc: &Location) {
        unsafe {
            gl::Uniform1i(loc.location(), *self);
        }
    }
}

impl Uniform for GLuint {
    fn uniform(&self, loc: &Location) {
        unsafe {
            gl::Uniform1ui(loc.location(), *self);
        }
    }
}

impl Uniform for Vector4<GLfloat> {
    fn uniform(&self, loc: &Location) {
        unsafe {
            let buf: &[GLfloat; 4] = self.as_ref();
            gl::Uniform4fv(loc.location(), 1, buf.as_ptr());
        }
    }
}

impl Uniform for Matrix4<GLfloat> {
    fn uniform(&self, loc: &Location) {
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
    fn uniform(&self, loc: &Location) {
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
    // sb_instance_buffer: Location,
}

impl DefaultLocations {
    pub fn new(program: &Program) -> Self {
        Self {
            projection:         Location::new(program, "_projection"),
            view:               Location::new(program, "_view"),
            model:              Location::new(program, "_model"),
            time:               Location::new(program, "_time"),
            flip_uvs:           Location::new(program, "_flip_uvs"),
            base_color:         Location::new(program, "_base_color"),
            tx_diffuse:         Location::new(program, "_tx_diffuse"),
            tx_normal:          Location::new(program, "_tx_normal"),
            // sb_instance_buffer: Location::new(program, "_sb_instance_buffer"),
        }
    }

    // TODO return ref?
    //   yea? because Uniform::uniform takes ref
    pub fn projection(&self) -> &Location {
        &self.projection
    }

    pub fn view(&self) -> &Location {
        &self.view
    }

    pub fn model(&self) -> &Location {
        &self.model
    }

    pub fn time(&self) -> &Location {
        &self.time
    }

    pub fn base_color(&self) -> &Location {
        &self.base_color
    }

    pub fn diffuse(&self) -> &Location {
        &self.tx_diffuse
    }

    pub fn normal(&self) -> &Location {
        &self.tx_normal
    }

    // pub fn sb_instance_buffer(&self) -> &Location {
        // &self.sb_instance_buffer
    // }
}

//

// texture region
// spritesheet
// bitmap font

//

#[derive(Debug)]
#[repr(C)]
pub struct SbParams {
    pub uv: Vector4<GLfloat>,
    pub transform: Transform2d,
    pub color: Vector4<GLfloat>,
}

impl Default for SbParams {
    fn default() -> Self {
        Self {
            uv: Vector4::new(0., 0., 1., 1.),
            transform: Default::default(),
            color: Vector4::new(1., 1., 1., 1.),
        }
    }
}

pub struct Spritebatch {
    buffer: InstanceBuffer<SbParams>,
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

        let vao = unsafe {
            mesh_quad.vao_mut()
        };

        let base = VertexAttribute {
            size: 4,
            ty: gl::FLOAT,
            normalized: false,
            stride: std::mem::size_of::<SbParams>(),
            offset: 0,
            divisor: 1,
        };

        // TODO use enum for all these
        vao.bind();
        ibo.bind_to(gl::ARRAY_BUFFER);
        vao.enable_attribute(3, VertexAttribute {
            offset: offset_of!(SbParams, uv),
            .. base
        });
        vao.enable_attribute(4, VertexAttribute {
            size: 2,
            offset: offset_of!(SbParams, transform) +
                    offset_of!(Transform2d, position),
            .. base
        });
        vao.enable_attribute(5, VertexAttribute {
            size: 2,
            offset: offset_of!(SbParams, transform) +
                    offset_of!(Transform2d, scale),
            .. base
        });
        vao.enable_attribute(6, VertexAttribute {
            size: 1,
            offset: offset_of!(SbParams, transform) +
                    offset_of!(Transform2d, rotation),
            .. base
        });
        vao.enable_attribute(7, VertexAttribute {
            offset: offset_of!(SbParams, color),
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
    pub fn pull(&mut self) -> &mut SbParams {
        if self.buffer.empty_count() == 0 {
            self.draw();
        }
        let ret = self.buffer.pull().unwrap();
        *ret = Default::default();
        ret
    }
}

//

// this is just a bunch of functions that generate uniforms on the fly
//   and applies them to current program
// also has meshes and textures

// poly
// line

pub struct Drawer {
    mesh_quad: Mesh,
    mesh_circle: Mesh,
    tex_white: Texture,
}

impl Drawer {
    pub fn new(circle_resolution: usize) -> Self {
        let img = RgbaImage::from_pixel(1, 1, Rgba::from([255, 255, 255, 255]));
        Self {
            mesh_quad: Mesh::new(Vertices::quad(false),
                                 gl::STATIC_DRAW,
                                 gl::TRIANGLE_STRIP),
            mesh_circle: Mesh::new(Vertices::circle(circle_resolution),
                                   gl::STATIC_DRAW,
                                   gl::TRIANGLE_FAN),
            tex_white: Texture::new(&img),
        }
    }

    pub fn sprite_px(&self, locations: &DefaultLocations, texture: &Texture, transform: &Transform2d) {
        TextureData::diffuse(texture).uniform(locations.diffuse());
        Matrix4::from_transform2d(transform).uniform(locations.model());
        self.mesh_quad.draw();
    }

    pub fn sprite(&self, locations: &DefaultLocations, texture: &Texture, transform: &Transform2d) {
        let temp = Transform2d {
            scale: Vector2::new(transform.scale.x * texture.width() as GLfloat,
                                transform.scale.y * texture.height() as GLfloat),
            .. *transform
        };
        self.sprite_px(locations, texture, &temp);
    }

    pub fn filled_rectangle(&self, locations: &DefaultLocations, rect: Vector4<GLfloat>) {
        let temp = Transform2d {
            position: Vector2::new(rect.x, rect.y),
            scale:    Vector2::new(rect.z - rect.x,
                                   rect.w - rect.y),
            rotation: 0.,
        };
        self.sprite_px(locations, &self.tex_white, &temp);
    }
}
