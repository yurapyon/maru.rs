#![allow(dead_code)]

use std::{
    mem,
    ops::Add
};

use cgmath::{
    prelude::*,
    Vector2,
    Vector3,
    Ortho,
    Matrix4,

    BaseNum,
};
use image::{
    self,
};
use gl::{
    self,
    types::*,
};
use num::{
    traits::AsPrimitive
};

//

use crate::gfx::{
    Texture,
};

//

// TODO where to use GLfloat or just use a rust float
//   could just put some compiler error is GLfloat != f32

//

/// Just a color.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Color {
    pub r: GLfloat,
    pub g: GLfloat,
    pub b: GLfloat,
    pub a: GLfloat,
}

impl Color {
    pub fn new_rgba(r: GLfloat, g: GLfloat, b: GLfloat, a:GLfloat) -> Self {
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

impl AsRef<[GLfloat; 4]> for Color {
    fn as_ref(&self) -> &[GLfloat; 4] {
        unsafe {
            mem::transmute(self)
        }
    }
}

//

/// An AABB rectangle.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct AABB<T> {
    pub x1: T,
    pub y1: T,
    pub x2: T,
    pub y2: T,
}

impl<T> AABB<T> {
    pub fn new(x1: T, y1: T, x2: T, y2: T) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
        }
    }
}

impl<T: BaseNum> AABB<T> {
    pub fn width(&self) -> T {
        self.x2 - self.x1
    }

    pub fn height(&self) -> T {
        self.y2 - self.y1
    }
}

impl<T: BaseNum + AsPrimitive<GLfloat>> AABB<T> {
    /// Return an `AABB<GLfloat>` normalized to the texture width and height.
    pub fn normalized(&self, tex: &Texture) -> AABB<GLfloat> {
        let fl_w = tex.width() as GLfloat;
        let fl_h = tex.height() as GLfloat;
        AABB::new(
            self.x1.as_() / fl_w,
            self.y1.as_() / fl_h,
            self.x2.as_() / fl_w,
            self.y2.as_() / fl_h,
        )
    }
}

impl<T: BaseNum> Add for AABB<T> {
    type Output = Self;

    fn add(self, other: Self::Output) -> Self {
        Self::new(self.x1 + other.x1,
                  self.y1 + other.y1,
                  self.x2 + other.x2,
                  self.y2 + other.y2,)
    }
}

impl<T: BaseNum + Zero> Zero for AABB<T> {
    fn zero() -> Self {
        Self::new(Zero::zero(),
                  Zero::zero(),
                  Zero::zero(),
                  Zero::zero(),)
    }

    fn is_zero(&self) -> bool {
        self.x1 == Zero::zero() &&
        self.y1 == Zero::zero() &&
        self.x2 == Zero::zero() &&
        self.y2 == Zero::zero()
    }
}

//

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    pub position: Vector3<GLfloat>,
    pub normal: Vector3<GLfloat>,
    pub uv: Vector2<GLfloat>,
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: Vector3::new(0., 0., 0.),
            normal:   Vector3::new(0., 0., 0.),
            uv:       Vector2::new(0., 0.),
        }
    }
}

pub struct Vertices {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<GLuint>,
}

impl Vertices {
    pub fn quad(centered: bool) -> Vertices {
        let mut vertices = Vec::with_capacity(4);

        vertices.push(Vertex {
            position: Vector3::new(1., 1., 0.),
            uv:       Vector2::new(1., 1.),
            .. Default::default()
        });

        vertices.push(Vertex {
            position: Vector3::new(1., 0., 0.),
            uv:       Vector2::new(1., 0.),
            .. Default::default()
        });

        vertices.push(Vertex {
            position: Vector3::new(0., 1., 0.),
            uv:       Vector2::new(0., 1.),
            .. Default::default()
        });

        vertices.push(Vertex {
            position: Vector3::new(0., 0., 0.),
            uv:       Vector2::new(0., 0.),
            .. Default::default()
        });

        if centered {
            for vert in vertices.iter_mut() {
                vert.position.x -= 0.5;
                vert.position.y -= 0.5;
            }
        }

        Self {
            vertices,
            indices: Vec::new(),
        }
    }

    pub fn circle(resolution: usize) -> Vertices {
        use std::f32::consts;

        let mut vertices = Vec::new();

        let angle_step = (consts::PI * 2.) / (resolution as GLfloat);

        for i in 0..resolution {
            let at = (i as f32) * angle_step;
            let x = at.cos() / 2.;
            let y = at.sin() / 2.;
            vertices.push(Vertex {
                position: Vector3::new(x, y, 0.),
                uv:       Vector2::new(x + 0.5, y + 0.5),
                .. Default::default()
            });
        }

        Self {
            vertices,
            indices: Vec::new(),
        }
    }
}

// TODO eq derives
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Transform2d {
    pub position: Vector2<GLfloat>,
    pub scale: Vector2<GLfloat>,
    pub rotation: GLfloat,
}

impl Transform2d {
    pub fn new(x: GLfloat, y: GLfloat, sx: GLfloat, sy: GLfloat, r: GLfloat) -> Self {
        Self {
            position: Vector2::new(x, y),
            scale: Vector2::new(sx, sy),
            rotation: r,
        }
    }
}

impl Default for Transform2d {
    fn default() -> Self {
        Self {
            position: Vector2::new(0., 0.),
            scale:    Vector2::new(1., 1.),
            rotation: 0.,
        }
    }
}

//

pub mod ext {
    use super::*;

    pub trait OrthoExt<S> {
        fn screen(width: u32, height: u32) -> Ortho<S>;
    }

    impl OrthoExt<GLfloat> for Ortho<GLfloat> {
        fn screen(width: u32, height: u32) -> Self {
            Self {
                left:   0.,
                right:  width as GLfloat,
                top:    0.,
                bottom: height as GLfloat,
                near:   -1.,
                far:    1.,
            }
        }
    }

    //

    // TODO use From<> trait
    pub trait Matrix4Ext<S> {
        fn from_transform2d(t2d: &Transform2d) -> Matrix4<S>;
    }

    impl Matrix4Ext<GLfloat> for Matrix4<GLfloat> {
        fn from_transform2d(t2d: &Transform2d) -> Self {
            let mut ret = Self::identity();
            let sx = t2d.scale.x;
            let sy = t2d.scale.y;
            let rc = t2d.rotation.cos();
            let rs = t2d.rotation.sin();
            ret[0][0] =  rc * sx;
            ret[0][1] =  rs * sx;
            ret[1][0] = -rs * sy;
            ret[1][1] =  rc * sy;
            ret[3][0] = t2d.position.x;
            ret[3][1] = t2d.position.y;
            ret
        }
    }
}
