mod color;
pub use color::*;

mod aabb;
pub use aabb::*;

mod transform2d;
pub use transform2d::*;

//

use nalgebra_glm as glm;

pub mod ext {
    use super::*;

    pub fn ortho_screen(dimensions: glm::UVec2) -> glm::Mat3 {
        let mut ret = glm::Mat3::identity();
        ret[(0, 0)] =  2. / dimensions.x as f32;
        ret[(1, 1)] = -2. / dimensions.y as f32;
        ret[(0, 2)] = -1.;
        ret[(1, 2)] =  1.;
        ret
    }

    //

    impl From<Transform2d> for glm::Mat3 {
        fn from(t2d: Transform2d) -> Self {
            let mut ret = Self::identity();
            let sx = t2d.scale.x;
            let sy = t2d.scale.y;
            let rc = t2d.rotation.cos();
            let rs = t2d.rotation.sin();
            ret[(0, 0)] =  rc * sx;
            ret[(1, 0)] =  rs * sx;
            ret[(0, 1)] = -rs * sy;
            ret[(1, 1)] =  rc * sy;
            ret[(0, 2)] = t2d.position.x;
            ret[(1, 2)] = t2d.position.y;
            ret
        }
    }
}
