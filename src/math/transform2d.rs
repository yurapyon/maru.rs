use nalgebra_glm as glm;

//

// TODO eq derives
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
}
