use image::{
    Rgba,
};

//

/// Just a color.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new_rgba(r: f32, g: f32, b: f32, a:f32) -> Self {
        Self {
            r,
            g,
            b,
            a,
        }
    }

    // TODO new_hsv

    pub fn white() -> Self {
        Self::new_rgba(1.0, 1.0, 1.0, 1.0)
    }

    pub fn black() -> Self {
        Self::new_rgba(0.0, 0.0, 0.0, 1.0)
    }
}

impl From<Color> for [u8; 4] {
    fn from(color: Color) -> Self {
        let r = (color.r * 255.).floor() as u8;
        let g = (color.g * 255.).floor() as u8;
        let b = (color.b * 255.).floor() as u8;
        let a = (color.a * 255.).floor() as u8;
        [r, g, b, a]
    }
}

impl From<Color> for Rgba<u8> {
    fn from(color: Color) -> Self {
        Self::from(<[u8; 4]>::from(color))
    }
}

impl AsRef<[f32; 4]> for Color {
    fn as_ref(&self) -> &[f32; 4] {
        use std::mem;

        unsafe {
            mem::transmute(self)
        }
    }
}
