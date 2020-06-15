#![allow(dead_code)]

use cgmath::{
    Vector2,
    Vector3,
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

pub struct Vertices {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<GLuint>,
}

