#![allow(dead_code)]

use std::default::Default;

use cgmath::{
    prelude::*,
    Vector2,
    Vector3,
    Ortho,
    Matrix4,
};
use gl;
use gl::types::*;

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

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Transform2d {
    pub position: Vector2<GLfloat>,
    pub scale: Vector2<GLfloat>,
    pub rotation: GLfloat,
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
        fn screen(width: S, height: S) -> Ortho<S>;
    }

    impl OrthoExt<GLfloat> for Ortho<GLfloat> {
        fn screen(width: GLfloat, height: GLfloat) -> Self {
            Self {
                left:   0.,
                right:  width,
                top:    0.,
                bottom: height,
                near:   -1.,
                far:    1.,
            }
        }
    }

    //

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
