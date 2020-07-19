use nalgebra_glm as glm;

// TODO shear
//      take transform2d (if still going to use it)
//        just turn into coordTransforms and push them
//      take entire mat3s?
// nalgabra has alot of stuff to handle transforms
//   could just have a stack of various affine transforms
//   Affine2

pub enum CoordinateTransform {
    Translate(glm::Vec2),
    Scale(glm::Vec2),
    Rotate(f32),
}

/// Draw with offsets.
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

    pub fn clear(&mut self) -> &glm::Mat3 {
        self.stk.clear();
        self.composed = glm::Mat3::identity();
        &self.composed
    }

    pub fn push(&mut self, t: CoordinateTransform) -> &glm::Mat3 {
        let temp = match t {
            // TODO allow these to be applied to mat3 directly
            //        maybe nalgebra already has stuff like this
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
        self.stk.push(t);
        &self.composed
    }

    /// Will panic if the stack is empty.
    pub fn pop(&mut self) -> &glm::Mat3 {
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
            },
            None => {
                panic!("coordinate stack underflow");
            },
        }
        &self.composed
    }
}
