use gl::{
    self,
    types::*,
};
use image::{
    RgbaImage,
};

//

// TODO
//   buffer data
/// Simple wrapper around an OpenGL texture.
#[derive(Debug)]
pub struct Texture {
    texture: GLuint,
    width: i32,
    height: i32,
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
                width: image.width() as i32,
                height: image.height() as i32,
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

    fn _get_dimensions(texture: GLuint, level: GLint) -> (i32, i32) {
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

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn dimensions(&self) -> (i32, i32) {
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
