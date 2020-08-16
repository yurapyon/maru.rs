use gl::{
    self,
    types::*,
};
use image::{
    Rgba,
    RgbaImage,
};

use super::Texture;

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
