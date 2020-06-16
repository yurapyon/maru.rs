#![allow(dead_code)]

use std::{
    ffi::CString,
    ptr,
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

// TODO use string?
//      can just use &str maybe
//        then have to worry abt lifetimes
#[derive(Debug)]
pub enum GfxError {
     BadInit(String),
}

//

pub struct ShaderTemplate {
    header: String,
    extras: String,
    effect: String,
    footer: String,
}

impl ShaderTemplate {
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
#[derive(Debug)]
pub struct Shader {
    shader: GLuint,
}

impl Shader {
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

#[derive(Debug)]
pub struct Program {
    program: GLuint,
}

impl Program {
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

    pub fn gl(&self) -> GLuint {
        self.program
    }

    pub fn gl_use(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
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

pub struct ProgramGenerator {
}

//

// TODO
//   if local image is needed later, just add it
//   buffer data
//   switch width and height to usizes?
pub struct Texture {
    texture: GLuint,
    width: GLint,
    height: GLint,
}

impl Texture {
    pub fn new(image: &RgbaImage) -> Result<Self, GfxError> {
        unsafe {
            // TODO check errors
            //      use glpixelstore instead
            // https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glPixelStore.xhtml
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
            Ok(ret)
        }
    }

    pub fn from_gl_texture(texture: GLuint) -> Result<Self, GfxError> {
        let (width, height) = Self::_get_dimensions(texture, 1);
        Ok(Self {
            texture,
            width,
            height,
        })
    }

    pub fn set_wrap(&mut self, s: GLenum, t: GLenum) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, s as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, t as GLint);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn set_filter(&mut self, min: GLenum, mag: GLenum) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag as GLint);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

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

    pub fn get_dimensions(&self, level: GLint) -> (GLint, GLint) {
        Self::_get_dimensions(self.texture, level)
    }

    pub fn gl(&self) -> GLuint {
        self.texture
    }

    pub fn width(&self) -> GLint {
        self.width
    }

    pub fn height(&self) -> GLint {
        self.height
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

struct TextureBuffer<T> {
    tbo: GLuint,
    texture: Texture,
    index: usize,
    buffer: Vec<T>,
}

impl<T> TextureBuffer<T> {
    pub fn new(len: usize) -> Result<Self, GfxError> {
        unsafe {
            let mut tbo = 0;
            gl::GenBuffers(1, &mut tbo);
            gl::BindBuffer(gl::TEXTURE_BUFFER, tbo);
            gl::BufferData(gl::TEXTURE_BUFFER,
                (len * mem::size_of::<T>()) as GLsizeiptr,
                ptr::null(),
                gl::STREAM_DRAW);

            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_BUFFER, texture);
            gl::TexBuffer(gl::TEXTURE_BUFFER, gl::RGBA32F, tbo);

            gl::BindTexture(gl::TEXTURE_BUFFER, 0);
            gl::BindBuffer(gl::TEXTURE_BUFFER, 0);

            let mut buffer = Vec::with_capacity(len);
            buffer.set_len(len);

            Ok(Self {
                tbo,
                texture: Texture::from_gl_texture(texture)?,
                index: 0,
                buffer,
            })
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

    pub fn buffer_data(&mut self) {
        if self.index == 0 {
            return;
        }

        unsafe {
            gl::BindBuffer(gl::TEXTURE_BUFFER, self.tbo);
            gl::BufferSubData(gl::TEXTURE_BUFFER,
                0,
                (self.index * mem::size_of::<T>()) as GLsizeiptr,
                self.buffer.as_ptr() as _);
            gl::BindBuffer(gl::TEXTURE_BUFFER, 0);
        }
    }

    pub fn fill_count(&self) -> usize {
        self.index
    }

    pub fn empty_count(&self) -> usize {
        self.buffer.len() - self.index
    }
}

impl<T> Drop for TextureBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.tbo);
        }
    }
}

//

pub struct Canvas {
    texture: Texture,
    fbo: GLuint,
    rbo: GLuint,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Result<Self, GfxError> {
        let mut img = RgbaImage::new(width, height);
        for p in img.pixels_mut() {
            p[3] = 255;
        }

        let texture = Texture::new(&img)?;

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

            Ok(Self {
                texture,
                fbo,
                rbo,
            })
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
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    buffer_type: GLenum,
    draw_type: GLenum,
}

impl Mesh {
    pub fn new(vertices: Vertices, buffer_type: GLenum, draw_type: GLenum) -> Result<Self, GfxError> {
        unsafe {
            let mut vao = 0;
            let mut vbo = 0;
            let mut ebo = 0;

            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                (vertices.vertices.len() * mem::size_of::<Vertex>()) as isize,
                vertices.vertices.as_ptr() as _,
                buffer_type);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                (vertices.indices.len() * mem::size_of::<GLuint>()) as isize,
                vertices.indices.as_ptr() as _,
                buffer_type);

            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                offset_of!(Vertex, position) as _);

            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                offset_of!(Vertex, normal) as _);

            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                offset_of!(Vertex, uv) as _);

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);

            gl::BindVertexArray(0);

            Ok(Self {
                vertices,
                vao,
                vbo,
                ebo,
                buffer_type,
                draw_type,
            })
        }
    }

    pub fn draw(&self) {
        unsafe {
            let i_ct = self.vertices.indices.len();

            gl::BindVertexArray(self.vao);
            if i_ct == 0 {
                gl::DrawArrays(self.draw_type, 0, self.vertices.vertices.len() as GLint);
            } else {
                gl::DrawElements(self.draw_type, i_ct as GLint, gl::UNSIGNED_INT, ptr::null());
            }
            gl::BindVertexArray(0);
        }
    }

    pub fn draw_instanced(&self, n: GLint) {
        unsafe {
            let i_ct = self.vertices.indices.len();

            gl::BindVertexArray(self.vao);
            if i_ct == 0 {
                gl::DrawArraysInstanced(self.draw_type, 0, self.vertices.vertices.len() as GLint, n);
            } else {
                gl::DrawElementsInstanced(self.draw_type, i_ct as GLint, gl::UNSIGNED_INT, ptr::null(), n);
            }
            gl::BindVertexArray(0);
        }
    }

    // TODO buffer data
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.ebo);
            gl::DeleteBuffers(1, &mut self.vbo);
            gl::DeleteVertexArrays(1, &mut self.vao);
        }
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

// texture region
// spritesheet
// bitmap font

//

pub struct DefaultLocations {
    projection: Location,
    view: Location,
    model: Location,
    time: Location,
    flip_uvs: Location,
    base_color: Location,
    diffuse: Location,
    normal: Location,
    sb_instance_buffer: Location,
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
            diffuse:            Location::new(program, "_diffuse"),
            normal:             Location::new(program, "_normal"),
            sb_instance_buffer: Location::new(program, "_sb_instance_buffer"),
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
        &self.diffuse
    }

    pub fn normal(&self) -> &Location {
        &self.normal
    }

    pub fn sb_instance_buffer(&self) -> &Location {
        &self.sb_instance_buffer
    }
}

// this is just a bunch of functions that generate uniforms on the fly
//   and applies them to current program
// also has meshes and textures

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
                                 gl::TRIANGLE_STRIP).unwrap(),
            mesh_circle: Mesh::new(Vertices::circle(circle_resolution),
                                   gl::STATIC_DRAW,
                                   gl::TRIANGLE_FAN).unwrap(),
            tex_white: Texture::new(&img).unwrap(),
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

// default uniforms
// default prog_gen
//   prog_gen
//   maru_gen_load_filepath

// spritebatch
// shapedrawer
