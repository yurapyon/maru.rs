use nalgebra_glm as glm;

use crate::{
    gfx::{
        DefaultLocations,
    },
};

// TODO could generalize this to 3d
// or not
pub enum CoordinateTransform {
    Translate(glm::Vec2),
    Scale(glm::Vec2),
    Rotate(f32),
}

/// Draw with offsets.
/// Uses the view matrix of the locations.
pub struct CoordinateStack {
    stk: Vec<CoordinateTransform>,
    composed: glm::Mat4,
}

impl CoordinateStack {
    pub fn with_capacity(size: usize) -> Self {
        let stk = Vec::with_capacity(size);
        let composed = glm::Mat4::identity();
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
        self.composed = glm::Mat4::identity();
        self.on_changed(locs);
    }

    pub fn push(&mut self, t: CoordinateTransform, locs: &DefaultLocations) {
        let temp = match t {
            CoordinateTransform::Translate(v) => {
                glm::translation(&glm::vec2_to_vec3(&v))
            },
            CoordinateTransform::Scale(v) => {
                let scl = glm::vec3(v.x, v.y, 1.);
                glm::scaling(&scl)
            },
            CoordinateTransform::Rotate(r) => {
                glm::rotation(r, &glm::vec3(0., 0., 1.))
            },
        };
        self.composed = temp * self.composed;
        /*
        self.composed = match t {
            CoordinateTransform::Translate(v) => {
                glm::translate(&self.composed, &glm::vec2_to_vec3(&v))
            },
            CoordinateTransform::Scale(v) => {
                let scl = glm::vec3(v.x, v.y, 1.);
                glm::scale(&self.composed, &scl)
            },
            CoordinateTransform::Rotate(r) => {
                glm::rotate_z(&self.composed, r)
            },
        };
        */
        self.on_changed(locs);
        self.stk.push(t);
    }

    /// Will panic if the stack is empty.
    pub fn pop(&mut self, locs: &DefaultLocations) {
        match self.stk.pop() {
            Some(t) => {

                let temp = match t {
                    CoordinateTransform::Translate(v) => {
                        glm::translation(&glm::vec2_to_vec3(&-v))
                    },
                    CoordinateTransform::Scale(v) => {
                        let scl = glm::vec3(v.x, v.y, 1.).map(f32::recip);
                        glm::scaling(&scl)
                    },
                    CoordinateTransform::Rotate(r) => {
                        glm::rotation(r, &glm::vec3(0., 0., -1.))
                    },
                };
                self.composed = temp * self.composed;
                /*
                self.composed = match t {
                    CoordinateTransform::Translate(v) => {
                        glm::translate(&self.composed, &glm::vec2_to_vec3(&-v))
                    },
                    CoordinateTransform::Scale(v) => {
                        let scl = glm::vec3(v.x, v.y, 1.).map(f32::recip);
                        glm::scale(&self.composed, &scl)
                    },
                    CoordinateTransform::Rotate(r) => {
                        glm::rotate_z(&self.composed, -r)
                    },
                };
                */
                self.on_changed(locs);
            },
            None => {
                panic!("coordinate stack underflow");
            },
        }
    }
}
