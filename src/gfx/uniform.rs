use gl::{
    self,
    types::*,
};
use nalgebra_glm as glm;

use crate::math::{
    Color,
};

use super::{
    Location,
    Texture,
};

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
