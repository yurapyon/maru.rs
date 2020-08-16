use std::mem;

use memoffset::offset_of;
use nalgebra_glm as glm;

use crate::{
    gfx::{
        Mesh,
        VertexArray,
        VertexAttribute,
        Vertex,
    },
};

//

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex2d {
    pub position: glm::Vec2,
    pub uv: glm::Vec2,
}

impl Vertex2d {
    pub fn quad(centered: bool) -> Vec<Self> {
        let mut ret = Vec::with_capacity(4);

        ret.push(Self {
            position: glm::vec2(1., 1.),
            uv:       glm::vec2(1., 1.),
        });

        ret.push(Self {
            position: glm::vec2(1., 0.),
            uv:       glm::vec2(1., 0.),
        });

        ret.push(Self {
            position: glm::vec2(0., 1.),
            uv:       glm::vec2(0., 1.),
        });

        ret.push(Self {
            position: glm::vec2(0., 0.),
            uv:       glm::vec2(0., 0.),
        });

        if centered {
            for vert in ret.iter_mut() {
                vert.position.x -= 0.5;
                vert.position.y -= 0.5;
            }
        }

        ret
    }

    pub fn circle(resolution: usize) -> Vec<Self> {
        use std::f32::consts;

        let mut ret = Vec::new();

        let angle_step = (consts::PI * 2.) / (resolution as f32);

        for i in 0..resolution {
            let at = (i as f32) * angle_step;
            let x = at.cos() / 2.;
            let y = at.sin() / 2.;
            ret.push(Self {
                position: glm::vec2(x, y),
                uv:       glm::vec2(x + 0.5, y + 0.5),
            });
        }

        ret
    }
}

impl Vertex for Vertex2d {
    fn set_attributes(vao: &mut VertexArray) {
        let base = VertexAttribute {
            size: 2,
            ty: gl::FLOAT,
            normalized: false,
            stride: mem::size_of::<Self>(),
            offset: offset_of!(Self, position),
            divisor: 0,
        };

        // TODO enum, move these out of here?
        vao.enable_attribute(0, VertexAttribute {
            offset: offset_of!(Self, position),
            .. base
        });
        vao.enable_attribute(1, VertexAttribute {
            offset: offset_of!(Self, uv),
            .. base
        });
    }
}

pub type Mesh2d = Mesh<Vertex2d>;
