// #![allow(dead_code)]

use image::{
    self,
};
use nalgebra_glm as glm;
use nalgebra::{
    Scalar,
    ClosedAdd,
    ClosedSub,
};
use num_traits::{
    ToPrimitive,
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

impl From<Color> for image::Rgba<u8> {
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

//

#[derive(Copy, Clone, Debug)]
#[repr(C)]
/// An AABB rectangle.
pub struct AABB<T: Scalar> {
    pub c1: glm::TVec2<T>,
    pub c2: glm::TVec2<T>,
}

impl<T: Scalar> AABB<T> {
    pub fn new(x1: T, y1: T, x2: T, y2: T) -> Self {
        Self {
            c1: glm::vec2(x1, y1),
            c2: glm::vec2(x2, y2),
        }
    }
}

impl<T: Scalar + ClosedSub + Copy> AABB<T> {
    // TODO height and width should do absolute value ?
    #[inline]
    pub fn width(&self) -> T {
        self.c2.x - self.c1.x
    }

    #[inline]
    pub fn height(&self) -> T {
        self.c2.y - self.c1.y
    }
}

impl<T: Scalar + ToPrimitive> AABB<T> {
    pub fn normalized(&self, vec: glm::TVec2<T>) -> AABB<f32> {
        let x = vec.x.to_f32().unwrap();
        let y = vec.y.to_f32().unwrap();
        let x1 = self.c1.x.to_f32().unwrap() / x;
        let y1 = self.c1.y.to_f32().unwrap() / y;
        let x2 = self.c2.x.to_f32().unwrap() / x;
        let y2 = self.c2.y.to_f32().unwrap() / y;
        AABB::new(x1, y1, x2, y2)
    }
}

impl<T: Scalar + PartialOrd> AABB<T> {
    pub fn reorient(&mut self) {
        use std::mem;

        if self.c1.x > self.c2.x {
            mem::swap(&mut self.c1.x, &mut self.c2.x);
        };

        if self.c1.y > self.c2.y {
            mem::swap(&mut self.c1.y, &mut self.c2.y);
        }
    }
}

impl<T: Scalar + ClosedAdd + Copy> AABB<T> {
    pub fn displace(&mut self, offset: glm::TVec2<T>) {
        self.c1 += offset;
        self.c2 += offset;
    }
}

/*
/// Slices to a vec as to be less complex
impl<T: Scalar + ClosedAdd + PartialOrd + Copy> AABB<T> {
    pub fn slice_up(&self, width: T, height: T) -> Vec<AABB<T>> {
        if self.c2.y <= self.c1.y ||
            self.c2.x <= self.c1.x {
            panic!("AABB must be normalized to take make slices of it");
        }

        let mut regions = Vec::new();

        let mut y = self.c1.y;
        while y <= self.c2.y {
            let mut x = self.c1.x;
            while x <= self.c2.x {
                regions.push(AABB::new(x, y,
                                       x + width,
                                       y + height));
                x += width;
            }
            y += height;
        }

        // TODO panic of slices are night aligned properly
        //   right now just ignores them

        regions
    }
}
*/

//

// TODO eq derives
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Transform2d {
    pub position: glm::Vec2,
    pub scale: glm::Vec2,
    pub rotation: f32,
}

impl Transform2d {
    pub fn new(x: f32, y: f32, sx: f32, sy: f32, r: f32) -> Self {
        Self {
            position: glm::vec2(x, y),
            scale:    glm::vec2(sx, sy),
            rotation: r,
        }
    }

    pub fn identity() -> Self {
        Self {
            position: glm::Vec2::zeros(),
            scale:    glm::Vec2::repeat(1.),
            rotation: 0.,
        }
    }
}

//

pub mod ext {
    use super::*;

    pub fn ortho_screen(dimensions: glm::UVec2) -> glm::Mat3 {
        let mut ret = glm::Mat3::identity();
        ret[(0, 0)] =  2. / dimensions.x as f32;
        ret[(1, 1)] = -2. / dimensions.y as f32;
        ret[(0, 2)] = -1.;
        ret[(1, 2)] =  1.;
        ret
    }

    //

    impl From<Transform2d> for glm::Mat3 {
        fn from(t2d: Transform2d) -> Self {
            let mut ret = Self::identity();
            let sx = t2d.scale.x;
            let sy = t2d.scale.y;
            let rc = t2d.rotation.cos();
            let rs = t2d.rotation.sin();
            ret[(0, 0)] =  rc * sx;
            ret[(1, 0)] =  rs * sx;
            ret[(0, 1)] = -rs * sy;
            ret[(1, 1)] =  rc * sy;
            ret[(0, 2)] = t2d.position.x;
            ret[(1, 2)] = t2d.position.y;
            ret
        }
    }
}
