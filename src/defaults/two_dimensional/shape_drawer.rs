use image::{
    RgbaImage,
    Rgba,
};
use nalgebra_glm as glm;

use crate::{
    gfx::{
        Texture,
    },
    math::{
        Transform2d,
    },
};

use super::{
    Mesh2d,
    Vertex2d,
    Locations,
};

//

// polygon
// line
// triangle

// lined rect

pub struct ShapeDrawer {
    line_thickness: f32,

    mesh_quad: Mesh2d,
    mesh_circle: Mesh2d,
    tex_white: Texture,
}

impl ShapeDrawer {
    pub fn new(circle_resolution: usize) -> Self {
        let white = RgbaImage::from_pixel(1, 1, Rgba::from([255, 255, 255, 255]));
        Self {
            mesh_quad: Mesh2d::new(Vertex2d::quad(false),
                                   Vec::new(),
                                   gl::STATIC_DRAW,
                                   gl::TRIANGLE_STRIP),
            mesh_circle: Mesh2d::new(Vertex2d::circle(circle_resolution),
                                     Vec::new(),
                                     gl::STATIC_DRAW,
                                     gl::TRIANGLE_FAN),
            tex_white: Texture::new(&white),
            line_thickness: 2.0,
        }
    }

    // TODO clean up api in general
    pub fn draw_quad(&self) {
        self.mesh_quad.draw();
    }

    pub fn line_thickness_mut(&mut self) -> &mut f32 {
        &mut self.line_thickness
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn filled_rectangle(&self, locations: &Locations, x1: f32, y1: f32, x2: f32, y2: f32) {
        let temp = Transform2d {
            position: glm::vec2(x1, y1),
            scale:    glm::vec2(x2 - x1, y2 - y1),
            .. Transform2d::identity()
        };
        locations.set_sprite_px(&self.tex_white, &temp);
        self.mesh_quad.draw();
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn circle(&self, locations: &Locations, x: f32, y: f32, r: f32) {
        let temp = Transform2d {
            position: glm::vec2(x, y),
            scale:    glm::vec2(r, r),
            .. Transform2d::identity()
        };
        locations.set_sprite_px(&self.tex_white, &temp);
        self.mesh_circle.draw();
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn horizontal_line(&self, locations: &Locations, y: f32, x1: f32, x2: f32) {
        let y1 = y - self.line_thickness / 2.;
        let y2 = y + self.line_thickness / 2.;
        self.filled_rectangle(locations, x1, y1, x2, y2);
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn vertical_line(&self, locations: &Locations, x: f32, y1: f32, y2: f32) {
        let x1 = x - self.line_thickness / 2.;
        let x2 = x + self.line_thickness / 2.;
        self.filled_rectangle(locations, x1, y1, x2, y2);
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn line(&self, locations: &Locations, x1: f32, y1: f32, x2: f32, y2: f32) {
        if x1 == x2 {
            self.vertical_line(locations, x1, y1, y2);
        } else if y1 == y2 {
            self.horizontal_line(locations, y1, x1, x2);
        } else {
            // TODO
        }
    }
}
