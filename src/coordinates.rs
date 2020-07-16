use nalgebra_glm as glm;

use crate::{
    gfx::{
        DefaultLocations,
    },
};

// TODO shear
//      take transform2d (if still going to use it)
//        just turn into coordTransforms and push them
//      take entire mat3s?

pub enum CoordinateTransform {
    Translate(glm::Vec2),
    Scale(glm::Vec2),
    Rotate(f32),
}

/// Draw with offsets.
/// Uses the view matrix of the locations.
pub struct CoordinateStack {
    stk: Vec<CoordinateTransform>,
    composed: glm::Mat3,
}

impl CoordinateStack {
    pub fn with_capacity(size: usize) -> Self {
        let stk = Vec::with_capacity(size);
        let composed = glm::Mat3::identity();
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
        self.composed = glm::Mat3::identity();
        self.on_changed(locs);
    }

    pub fn push(&mut self, t: CoordinateTransform, locs: &DefaultLocations) {
        let temp = match t {
            CoordinateTransform::Translate(v) => {
                glm::translation2d(&v)
            },
            CoordinateTransform::Scale(v) => {
                glm::scaling2d(&v)
            },
            CoordinateTransform::Rotate(r) => {
                glm::rotation2d(r)
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
                        glm::translation2d(&-v)
                    },
                    CoordinateTransform::Scale(v) => {
                        glm::scaling2d(&v.map(f32::recip))
                    },
                    CoordinateTransform::Rotate(r) => {
                        glm::rotation2d(-r)
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
