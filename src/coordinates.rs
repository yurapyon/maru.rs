use cgmath::{
    prelude::*,
    Vector2,
    Matrix4,
    Rad,
};

use crate::{
    gfx::{
        DefaultLocations,
    },
};

pub enum CoordinateTransform {
    Translate(Vector2<f32>),
    Scale(Vector2<f32>),
    Rotate(Rad<f32>),
}

/// Draw with offsets.
/// Uses the view matrix of the locations.
pub struct CoordinateStack {
    stk: Vec<CoordinateTransform>,
    composed: Matrix4<f32>,
}

impl CoordinateStack {
    pub fn with_capacity(size: usize) -> Self {
        let stk = Vec::with_capacity(size);
        let composed = Matrix4::identity();
        Self {
            stk,
            composed,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stk.is_empty()
    }

    fn on_changed(&mut self, locs: &DefaultLocations) {
        locs.view().set(&self.composed);
    }

    pub fn clear(&mut self, locs: &DefaultLocations) {
        self.stk.clear();
        self.composed = Matrix4::identity();
        self.on_changed(locs);
    }

    pub fn push(&mut self, t: CoordinateTransform, locs: &DefaultLocations) {
        let temp = match t {
            CoordinateTransform::Translate(v) => {
                Matrix4::from_translation(v.extend(0.))
            },
            CoordinateTransform::Scale(v) => {
                Matrix4::from_nonuniform_scale(v.x, v.y, 0.)
            },
            CoordinateTransform::Rotate(r) => {
                Matrix4::from_angle_z(r)
            },
        };
        self.composed = temp * self.composed;
        self.on_changed(locs);
        self.stk.push(t);
    }

    /// Will panic if the stack is empty.
    pub fn pop(&mut self, locs: &DefaultLocations) {
        match self.stk.pop() {
            Some(t) => {
                let temp = match t {
                    CoordinateTransform::Translate(v) => {
                        Matrix4::from_translation(-v.extend(0.))
                    },
                    CoordinateTransform::Scale(v) => {
                        Matrix4::from_nonuniform_scale(1. / v.x, 1. / v.y, 0.)
                    },
                    CoordinateTransform::Rotate(r) => {
                        Matrix4::from_angle_z(-r)
                    },
                };
                self.composed = temp * self.composed;
                self.on_changed(locs);
            },
            None => {
                panic!("coordinate stack underflow");
            },
        }
    }
}

