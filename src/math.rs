#![allow(dead_code)]

use std::{
    mem,
};

use cgmath::{
    prelude::*,
    Point2,
    Point3,
    Vector2,
    Vector3,
    Ortho,
    Matrix4,
    Rad,

    BaseNum,
};
use image::{
    self,
};

//

use crate::gfx::{
    Texture,
};

//

// note: GLfloats are always going to be f32
// https://www.khronos.org/opengl/wiki/OpenGL_Type

// TODO use cgmath the right way
// for now just use transform2d the way i have it
// someday try and use matrix3 instead with proper linear algebra stuff?

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
    pub corner1: Point2<T>,
    pub corner2: Point2<T>,
}

impl<T> AABB<T> {
    pub fn new(x1: T, y1: T, x2: T, y2: T) -> Self {
        Self {
            corner1: Point2::new(x1, y1),
            corner2: Point2::new(x2, y2),
        }
    }
}

impl<T: BaseNum> AABB<T> {
    pub fn width(&self) -> T {
        self.corner2.x - self.corner1.x
    }

    pub fn height(&self) -> T {
        self.corner2.y - self.corner1.y
    }

    /// Return an `AABB<f32>` normalized to the texture width and height.
    pub fn normalized(&self, tex: &Texture) -> AABB<f32> {
        let tx_point = Point2::new(tex.width() as f32, tex.height() as f32);
        let corner1 = self.corner1.cast().unwrap().div_element_wise(tx_point);
        let corner2 = self.corner2.cast().unwrap().div_element_wise(tx_point);
        AABB {
            corner1,
            corner2,
        }
    }
}

/*
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
*/

//

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
    pub uv: Point2<f32>,
}

impl Vertex {
    fn zero() -> Self {
        Self {
            position: Point3::new(0., 0., 0.),
            normal:   Vector3::new(0., 0., 0.),
            uv:       Point2::new(0., 0.),
        }
    }
}

pub struct Vertices {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Vertices {
    pub fn quad(centered: bool) -> Vertices {
        let mut vertices = Vec::with_capacity(4);

        vertices.push(Vertex {
            position: Point3::new(1., 1., 0.),
            uv:       Point2::new(1., 1.),
            .. Vertex::zero()
        });

        vertices.push(Vertex {
            position: Point3::new(1., 0., 0.),
            uv:       Point2::new(1., 0.),
            .. Vertex::zero()
        });

        vertices.push(Vertex {
            position: Point3::new(0., 1., 0.),
            uv:       Point2::new(0., 1.),
            .. Vertex::zero()
        });

        vertices.push(Vertex {
            position: Point3::new(0., 0., 0.),
            uv:       Point2::new(0., 0.),
            .. Vertex::zero()
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

        let angle_step = (consts::PI * 2.) / (resolution as f32);

        for i in 0..resolution {
            let at = (i as f32) * angle_step;
            let x = at.cos() / 2.;
            let y = at.sin() / 2.;
            vertices.push(Vertex {
                position: Point3::new(x, y, 0.),
                uv:       Point2::new(x + 0.5, y + 0.5),
                .. Vertex::zero()
            });
        }

        Self {
            vertices,
            indices: Vec::new(),
        }
    }
}

// TODO eq derives
//      rename default to identity
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Transform2d {
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation: Rad<f32>,
}

impl Transform2d {
    pub fn new(x: f32, y: f32, sx: f32, sy: f32, r: f32) -> Self {
        Self {
            position: Point2::new(x, y),
            scale: Vector2::new(sx, sy),
            rotation: Rad(r),
        }
    }

    /// Multiplicitave identity.
    pub fn identity() -> Self {
        Self {
            position: Point2::origin(),
            scale:    Vector2::new(1., 1.),
            rotation: Rad::zero(),
        }
    }

    /*
    pub fn translate(&mut self, x: f32, y: f32) {
        self.position.x += x;
        self.position.y += y;
    }

    pub fn scale_from_origin(&mut self, sx: f32, sy: f32) {
        self.position.x *= sx;
        self.position.y *= sy;
        self.scale.x *= sx;
        self.scale.y *= sy;
    }

    pub fn rotate_from_origin(&mut self, r: f32) {
        // self.position.x += x;
        // self.position.y += y;
        self.rotation += Rad(r);
    }
    */
}

//

pub mod ext {
    use super::*;

    pub trait OrthoExt<S> {
        fn screen(width: u32, height: u32) -> Ortho<S>;
    }

    impl OrthoExt<f32> for Ortho<f32> {
        fn screen(width: u32, height: u32) -> Self {
            Self {
                left:   0.,
                right:  width as f32,
                top:    0.,
                bottom: height as f32,
                near:   -1.,
                far:    1.,
            }
        }
    }

    //

    impl From<Transform2d> for Matrix4<f32> {
        fn from(t2d: Transform2d) -> Self {
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
