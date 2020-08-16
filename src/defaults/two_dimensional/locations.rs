use nalgebra_glm as glm;

use crate::{
    gfx::{
        Location,
        TextureData,
        Texture,
        Program,
    },
    math::{
        Transform2d,
        Color,
    },
};

//

pub struct Locations {
    screen: Location,
    view: Location,
    model: Location,
    time: Location,
    flip_uvs: Location,
    base_color: Location,
    tx_diffuse: Location,
    tx_normal: Location,
}

impl Locations {
    pub fn new(program: &Program) -> Self {
        Self {
            screen:     Location::new(program, "_screen"),
            view:       Location::new(program, "_view"),
            model:      Location::new(program, "_model"),
            time:       Location::new(program, "_time"),
            flip_uvs:   Location::new(program, "_flip_uvs"),
            base_color: Location::new(program, "_base_color"),
            tx_diffuse: Location::new(program, "_tx_diffuse"),
            tx_normal:  Location::new(program, "_tx_normal"),
        }
    }

    #[inline]
    pub fn screen(&self) -> &Location {
        &self.screen
    }

    #[inline]
    pub fn view(&self) -> &Location {
        &self.view
    }

    #[inline]
    pub fn model(&self) -> &Location {
        &self.model
    }

    #[inline]
    pub fn time(&self) -> &Location {
        &self.time
    }

    #[inline]
    pub fn flip_uvs(&self) -> &Location {
        &self.flip_uvs
    }

    #[inline]
    pub fn base_color(&self) -> &Location {
        &self.base_color
    }

    #[inline]
    pub fn diffuse(&self) -> &Location {
        &self.tx_diffuse
    }

    #[inline]
    pub fn normal(&self) -> &Location {
        &self.tx_normal
    }

    /// Note: does not set textures
    pub fn reset(&self) {
        let m3_iden = glm::Mat3::identity();
        self.screen().set(&m3_iden);
        self.view().set(&m3_iden);
        self.model().set(&m3_iden);
        self.base_color().set(&Color::white());
        self.flip_uvs().set(&false);
        self.time().set(&0.);
    }

    #[inline]
    pub fn set_rectangle(&self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let temp = Transform2d {
            position: glm::vec2(x1, y1),
            scale:    glm::vec2(x2 - x1, y2 - y1),
            .. Transform2d::identity()
        };
        self.model().set(&glm::Mat3::from(temp));
    }

    #[inline]
    pub fn set_sprite_px(&self, texture: &Texture, transform: &Transform2d) {
        self.diffuse().set(&TextureData::diffuse(texture));
        self.model().set(&glm::Mat3::from(*transform));
    }

    #[inline]
    pub fn set_sprite(&self, texture: &Texture, transform: &Transform2d) {
        let (w, h) = texture.dimensions();
        let temp = Transform2d {
            scale: glm::vec2(transform.scale.x * w as f32,
                             transform.scale.y * h as f32),
            .. *transform
        };
        self.set_sprite_px(texture, &temp);
    }

    #[inline]
    pub fn set_base_color_rgba(&self, r: f32, g: f32, b: f32, a: f32) {
        self.base_color.set(&Color::new_rgba(r, g, b, a));
    }
}

