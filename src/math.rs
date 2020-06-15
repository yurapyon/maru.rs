#![allow(dead_code)]

use std::default::Default;

use cgmath;
use gl;
use gl::types::*;

// TODO use GLfolat instead of f32?
// planning on using these with opengl directly

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

// column major
#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
pub struct Mat4 {
    data: [[f32; 4]; 4],
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

pub struct Vertices {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<GLuint>,
}

