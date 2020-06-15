#![allow(dead_code)]

use std::{
    ffi::CString,
    ptr,
    mem,
};

use cgmath;
use gl;
use gl::types::*;
use image;
use image::RgbaImage;
use memoffset::offset_of;

//

#[derive(Debug)]
pub enum GfxError {
     BadInit(String),
}

//

#[derive(Debug)]
pub struct Shader {
    shader: GLuint,
}

impl Shader {
    pub fn new(ty: GLenum, strings: &[&str]) -> Result<Self, GfxError> {
        let c_strs: Vec<_> = strings.iter()
            .map(| s | {
                CString::new(s.as_bytes()).unwrap()
            })
            .collect();

        let c_ptrs: Vec<_> = c_strs.iter()
            .map(| s | {
                s.as_ptr()
            })
            .collect();

        unsafe {
            let shader = gl::CreateShader(ty);

            gl::ShaderSource(shader, c_ptrs.len() as i32, c_ptrs.as_ptr(), ptr::null());
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
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}

//

pub struct Texture {
    texture: GLuint,
    image: RgbaImage,
}

impl Texture {
    pub fn new(image: RgbaImage) -> Result<Self, GfxError> {
        unsafe {
            // TODO check errors
            //      use glpixelstore instead
            // https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glPixelStore.xhtml
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            let ty = if cfg!(target_endian = "little") {
                    gl::UNSIGNED_INT_8_8_8_8
                } else {
                    gl::UNSIGNED_INT_8_8_8_8_REV
                };

            let i_vec = image.clone().into_vec();

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint,
                image.width() as GLint, image.height() as GLint, 0,
                gl::RGBA, ty, i_vec.as_ptr() as _);

            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            let mut ret = Self {
                texture,
                image,
            };

            ret.set_wrap(gl::REPEAT, gl::REPEAT);
            ret.set_filter(gl::LINEAR, gl::LINEAR);
            Ok(ret)
        }
    }

    pub fn from_gl_texture(texture: GLuint) -> Result<Self, GfxError> {
        Ok(Self {
            texture,
            image: RgbaImage::new(1, 1),
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

    pub fn gl(&self) -> GLuint {
        self.texture
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
                (len * mem::size_of::<T>()) as isize,
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
                texture: Texture::from_gl_texture(texture).unwrap(),
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
        // TODO
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

// canvas

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
                mem::size_of::<Vertex>() as i32,
                offset_of!(Vertex, position) as _);

            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE,
                mem::size_of::<Vertex>() as i32,
                offset_of!(Vertex, normal) as _);

            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE,
                mem::size_of::<Vertex>() as i32,
                offset_of!(Vertex, uv) as _);

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

// TODO
//   texture should be &'a Texture
//   use a separate TextureSlot struct just for textures
pub struct TextureData {
    select: GLenum,
    bind_to: GLenum,
    texture: GLuint,
}

pub enum UniformData {
    Float(f32),
    Bool(bool),
    Int(i32),
    Vec4(Vec4),
    Mat4(Mat4),
    TextureData(TextureData),
}

pub struct Uniform {
    location: GLint,
    data: UniformData,
}

impl Uniform {
    pub fn new(data: UniformData, program: &Program, name: &str) -> Result<Self, GfxError> {
        unsafe {
            // TODO error if not found
            //  location == -1
            let location = gl::GetUniformLocation(program.gl(), name.as_ptr() as _);
            Ok(Self {
                location,
                data,
            })
        }
    }

    pub fn apply(&self) {
        use UniformData::*;
        unsafe {
            match &self.data {
                Float(val) => gl::Uniform1f(self.location, *val),
                Bool(val)  => gl::Uniform1i(self.location, if *val { 1 } else { 0 }),
                Int(val)   => gl::Uniform1i(self.location, *val),
                Vec4(val)  => {
                    let mut buf: [GLfloat; 4] = [
                        val.x,
                        val.y,
                        val.z,
                        val.w,
                        ];
                    gl::Uniform4fv(self.location, 1, buf.as_ptr());
                },
                // TODO mat4
                _ => {}
            }
        }
    }
}
