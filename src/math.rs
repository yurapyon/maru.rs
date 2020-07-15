#![allow(dead_code)]

use std::{
    mem,
};

use approx::{
    AbsDiffEq
};
use image::{
    self,
};
use nalgebra_glm as glm;
use nalgebra::{
    Scalar,
    ClosedAdd,
    ClosedSub,
    ClosedMul,
};
use num_traits::{
    FromPrimitive,
    ToPrimitive,
    Bounded,
};

//

pub trait Number:
    Scalar
    + Copy
    + PartialOrd
    + ClosedAdd
    + ClosedSub
    + ClosedMul
    + AbsDiffEq<Epsilon = Self>
    + FromPrimitive
    + ToPrimitive
    + Bounded
{
}

impl<T> Number for T
where
    T: Scalar
       + Copy
       + PartialOrd
       + ClosedAdd
       + ClosedSub
       + ClosedMul
       + AbsDiffEq<Epsilon = Self>
       + FromPrimitive
       + ToPrimitive
       + Bounded
{
}

//

// note: GLfloats are always going to be f32
// https://www.khronos.org/opengl/wiki/OpenGL_Type

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
pub struct AABB<T: Number> {
    pub corner1: glm::TVec2<T>,
    pub corner2: glm::TVec2<T>,
}

impl<T: Number> AABB<T> {
    pub fn new(x1: T, y1: T, x2: T, y2: T) -> Self {
        Self {
            corner1: glm::vec2(x1, y1),
            corner2: glm::vec2(x2, y2),
        }
    }

    pub fn width(&self) -> T {
        self.corner2.x - self.corner1.x
    }

    pub fn height(&self) -> T {
        self.corner2.y - self.corner1.y
    }

    pub fn normalized(&self, point: &glm::TVec2<T>) -> AABB<f32> {
        let point = point.map(| x | x.to_f32().unwrap());
        let corner1 = self.corner1.map(| x | x.to_f32().unwrap()).component_div(&point);
        let corner2 = self.corner2.map(| x | x.to_f32().unwrap()).component_div(&point);
        AABB {
            corner1,
            corner2,
        }
    }
}


//

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub uv: glm::Vec2,
}

impl Vertex {
    fn zero() -> Self {
        Self {
            position: glm::vec3(0., 0., 0.),
            normal: glm::vec3(0., 0., 0.),
            uv: glm::vec2(0., 0.),
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
            position: glm::vec3(1., 1., 0.),
            uv:       glm::vec2(1., 1.),
            .. Vertex::zero()
        });

        vertices.push(Vertex {
            position: glm::vec3(1., 0., 0.),
            uv:       glm::vec2(1., 0.),
            .. Vertex::zero()
        });

        vertices.push(Vertex {
            position: glm::vec3(0., 1., 0.),
            uv:       glm::vec2(0., 1.),
            .. Vertex::zero()
        });

        vertices.push(Vertex {
            position: glm::vec3(0., 0., 0.),
            uv:       glm::vec2(0., 0.),
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
                position: glm::vec3(x, y, 0.),
                uv:       glm::vec2(x + 0.5, y + 0.5),
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

    pub fn ortho_screen(width: u32, height: u32) -> glm::Mat4 {
        glm::ortho(0., width as f32, height as f32, 0., -1., 1.)
    }

    //

    impl From<Transform2d> for glm::Mat4 {
        fn from(t2d: Transform2d) -> Self {
            let mut ret = Self::identity();
            let sx = t2d.scale.x;
            let sy = t2d.scale.y;
            let rc = t2d.rotation.cos();
            let rs = t2d.rotation.sin();
            ret[(0, 0)] =  rc * sx;
            ret[(0, 1)] =  rs * sx;
            ret[(1, 0)] = -rs * sy;
            ret[(1, 1)] =  rc * sy;
            ret[(3, 0)] = t2d.position.x;
            ret[(3, 1)] = t2d.position.y;
            ret
        }
    }
}
