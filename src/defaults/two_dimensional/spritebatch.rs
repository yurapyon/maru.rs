use memoffset::offset_of;

use crate::{
    gfx::{
        Instancer,
        BoundInstancer,
        VertexArray,
        VertexAttribute,
        Mesh,
        UvRegion,
        Vertex,
    },
    math::{
        Color,
        Transform2d,
    },
};

use super::{
    BitmapFont,
    Vertex2d,
    Mesh2d,
};

//

// TODO think about using mat3 instead of t2d
#[derive(Debug)]
#[repr(C)]
pub struct SbSprite {
    pub uv: UvRegion,
    pub transform: Transform2d,
    pub color: Color,
}

impl Default for SbSprite {
    fn default() -> Self {
        Self {
            uv: UvRegion::new(0., 0., 1., 1.),
            transform: Transform2d::identity(),
            color: Color::new_rgba(1., 1., 1., 1.),
        }
    }
}

impl Vertex for SbSprite {
    fn set_attributes(vao: &mut VertexArray) {
        let base = VertexAttribute {
            size: 4,
            ty: gl::FLOAT,
            normalized: false,
            stride: std::mem::size_of::<Self>(),
            offset: 0,
            divisor: 1,
        };

        // TODO use enum for all the attrib locs

        vao.enable_attribute(2, VertexAttribute {
            offset: offset_of!(Self, uv),
            .. base
        });
        vao.enable_attribute(3, VertexAttribute {
            size: 2,
            offset: offset_of!(Self, transform) +
                    offset_of!(Transform2d, position),
            .. base
        });
        vao.enable_attribute(4, VertexAttribute {
            size: 2,
            offset: offset_of!(Self, transform) +
                    offset_of!(Transform2d, scale),
            .. base
        });
        vao.enable_attribute(5, VertexAttribute {
            size: 1,
            offset: offset_of!(Self, transform) +
                    offset_of!(Transform2d, rotation),
            .. base
        });
        vao.enable_attribute(6, VertexAttribute {
            offset: offset_of!(Self, color),
            .. base
        });
    }
}

/// 2d instancer
pub struct Spritebatch {
    instancer: Instancer<SbSprite>,
    quad: Mesh<Vertex2d>,
    quad_centered: Mesh<Vertex2d>,
}

impl Spritebatch {
    pub fn new(size: usize) -> Self {
        let instancer = Instancer::new(size);
        let mut quad = Mesh2d::new(Vertex2d::quad(false),
                                   Vec::new(),
                                   gl::STATIC_DRAW,
                                   gl::TRIANGLE_STRIP);
        let mut quad_centered = Mesh2d::new(Vertex2d::quad(true),
                                            Vec::new(),
                                            gl::STATIC_DRAW,
                                            gl::TRIANGLE_STRIP);

        instancer.make_mesh_compatible(&mut quad);
        instancer.make_mesh_compatible(&mut quad_centered);

        Self {
            instancer,
            quad,
            quad_centered,
        }
    }

    pub fn bind<'a>(&'a mut self, centered_quad: bool) -> BoundInstancer<'a, SbSprite, Vertex2d> {
        self.instancer.bind(if centered_quad {
                &self.quad_centered
            } else {
                &self.quad
            })
    }
}

impl<'a> BoundInstancer<'a, SbSprite, Vertex2d> {
    pub fn print(&mut self, font: &BitmapFont, text: &str) {
        // TODO set font texture as diffuse
        //        cant do this without diffuse location

        let mut x = 0.;
        let font_h = font.texture().height() as f32;
        for ch in text.chars() {
            let region_w = font.region(ch).width() as f32;
            let sp = self.pull_default();
            sp.uv = font.uv_region(ch);
            sp.transform.position.x = x;
            sp.transform.scale.x = region_w;
            sp.transform.scale.y = font_h;
            x += region_w + 1.;
        }
    }
}
