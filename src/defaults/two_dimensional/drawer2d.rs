use image::{
    RgbaImage,
    Rgba,
};
use nalgebra_glm as glm;

use crate::{
    coordinates::{
        CoordinateStack,
    },
    gfx::{
        BoundInstancer,
        TextureData,
        Texture,
        UvRegion,
    },
    math::{
        ext::*,
        Color,
        Transform2d,
    },
};

use super::{
    SbSprite,
    Vertex2d,
    Spritebatch,
    Locations,
    BitmapFont,
    ShapeDrawer,
    Program2d,
};

#[doc(no_inline)]
pub use crate::coordinates::CoordinateTransform;

//

// TODO
// put this in global resources
// have a vec of owned canvases or something
// use this to bind and unbind canvases,
// refering to global ctx window width
/*
struct CanvasManager {
}
*/

//

pub struct DrawDefaults {
    pub program: Program2d,
    pub spritebatch_program: Program2d,
    pub white_texture: Texture,
}

impl DrawDefaults {
    pub fn new() -> Self {
        let program = Program2d::default_program();
        let spritebatch_program = Program2d::default_spritebatch_program();
        let white = RgbaImage::from_pixel(1, 1, Rgba::from([255, 255, 255, 255]));
        let white_texture = Texture::new(&white);

        Self {
            program,
            spritebatch_program,
            white_texture,
        }
    }
}

//

// TODO
//   manage canvases
//   keep a default prog and texture here so dont have to send them in on bind
//   api for bound spritebatch is kinda weird

pub struct Drawer2d {
    coord_stack: CoordinateStack,
    sprites: Spritebatch,
    shapes: ShapeDrawer,
}

impl Drawer2d {
    pub fn new() -> Self {
        let coord_stack = CoordinateStack::with_capacity(10);
        let sprites = Spritebatch::new(500);
        let shapes = ShapeDrawer::new(50);

        Self {
            coord_stack,
            sprites,
            shapes,
        }
    }

    pub fn bind_spritebatch<'a>(&'a mut self,
                                prog: &'a Program2d,
                                texture: &'a Texture,
                                canvas_dimensions: (u32, u32),
                                centered_quad: bool,
    ) -> BoundSpritebatch<'a> {
        BoundSpritebatch {
            base: BoundDrawer2d::new(&mut self.coord_stack, prog, texture, canvas_dimensions),
            sb: self.sprites.bind(centered_quad),
            sprite_color: Color::white(),
        }
    }

    pub fn bind_shape_drawer<'a>(&'a mut self,
                                 prog: &'a Program2d,
                                 texture: &'a Texture,
                                 canvas_dimensions: (u32, u32)
    ) -> BoundShapeDrawer<'a> {
        BoundShapeDrawer {
            base: BoundDrawer2d::new(&mut self.coord_stack, prog, texture, canvas_dimensions),
            drawer: &self.shapes,
        }
    }
}

//

// canvas manager goes here
pub struct BoundDrawer2d<'a> {
    coord_stack: &'a mut CoordinateStack,
    prog: &'a Program2d,
    texture: &'a Texture,
    canvas_width: u32,
    canvas_height: u32,
}

impl<'a> BoundDrawer2d<'a> {
    pub fn new(coord_stack: &'a mut CoordinateStack,
               prog: &'a Program2d,
               texture: &'a Texture,
               canvas_dimensions: (u32, u32)
    ) -> Self {
        let mut ret = Self {
            coord_stack,
            prog,
            texture,
            canvas_width: canvas_dimensions.0,
            canvas_height: canvas_dimensions.1,
        };
        ret.init_program();
        ret.init_texture();
        ret
    }

    pub fn set_program(&mut self, prog: &'a Program2d) {
        self.prog = prog;
        self.init_program();
    }

    fn init_program(&mut self) {
        self.prog.prog.bind();
        self.prog.locs.reset();
        self.prog.locs.screen().set(&ortho_screen(glm::vec2(self.canvas_width, self.canvas_height)));
        self.prog.locs.view().set(self.coord_stack.clear());
    }

    pub fn set_texture(&mut self, texture: &'a Texture) {
        self.texture = texture;
        self.init_texture();
    }

    fn init_texture(&mut self) {
        self.prog.locs.diffuse().set(&TextureData::diffuse(self.texture));
    }

    pub fn push_coord(&mut self, t: CoordinateTransform) {
        self.prog.locs.view().set(self.coord_stack.push(t));
    }

    pub fn pop_coord(&mut self) {
        if !self.coord_stack.is_empty() {
            self.prog.locs.view().set(self.coord_stack.pop());
        }
    }

    pub fn locations(&mut self) -> &Locations {
        &self.prog.locs
    }
}

//

pub struct BoundSpritebatch<'a> {
    base: BoundDrawer2d<'a>,
    sb: BoundInstancer<'a, SbSprite, Vertex2d>,

    sprite_color: Color,
}

impl<'a> BoundSpritebatch<'a> {
    // 'inherited'
    pub fn set_program(&mut self, prog: &'a Program2d) {
        use std::ptr;

        if !ptr::eq(self.base.prog, prog) {
            self.draw_now();
        }
        self.base.set_program(prog);
    }

    pub fn set_texture(&mut self, texture: &'a Texture) {
        use std::ptr;

        if !ptr::eq(self.base.texture, texture) {
            self.draw_now();
        }
        self.base.set_texture(texture);
    }

    pub fn push_coord(&mut self, t: CoordinateTransform) {
        self.draw_now();
        self.base.push_coord(t);
    }

    pub fn pop_coord(&mut self) {
        self.draw_now();
        self.base.pop_coord();
    }

    pub fn locations(&mut self) -> &Locations {
        self.base.locations()
    }

    //

    pub fn draw_now(&mut self) {
        self.sb.draw();
    }

    // TODO cleanup this api
    pub fn set_sprite_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.sprite_color.r = r;
        self.sprite_color.g = g;
        self.sprite_color.b = b;
        self.sprite_color.a = a;
    }

    pub fn region(&mut self, uv: &UvRegion, x1: f32, y1: f32, x2: f32, y2: f32) {
        let sprite = self.sb.pull_default();
        sprite.color = self.sprite_color;
        sprite.uv = *uv;
        sprite.transform.position.x = x1;
        sprite.transform.position.y = y1;
        sprite.transform.scale.x = x2 - x1;
        sprite.transform.scale.y = y2 - y1;
    }

    pub fn rectangle(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let sprite = self.sb.pull_default();
        sprite.color = self.sprite_color;
        sprite.transform.position.x = x1;
        sprite.transform.position.y = y1;
        sprite.transform.scale.x = x2 - x1;
        sprite.transform.scale.y = y2 - y1;
    }

    pub fn print(&mut self, font: &'a BitmapFont, text: &str) {
        self.base.set_texture(font.texture());
        self.sb.print(font, text);
        self.draw_now();
    }

    pub fn sprite(&mut self, transform: Transform2d, uv: UvRegion, color: Color) {
        let sprite = self.sb.pull();
        sprite.transform = transform;
        sprite.uv = uv;
        sprite.color = color;
    }
}

//

pub struct BoundShapeDrawer<'a> {
    base: BoundDrawer2d<'a>,
    drawer: &'a ShapeDrawer,
}

impl<'a> BoundShapeDrawer<'a> {
    // 'inherited'
    pub fn set_program(&mut self, prog: &'a Program2d) {
        self.base.set_program(prog);
    }

    pub fn set_texture(&mut self, texture: &'a Texture) {
        self.base.set_texture(texture);
    }

    pub fn push_coord(&mut self, t: CoordinateTransform) {
        self.base.push_coord(t);
    }

    pub fn pop_coord(&mut self) {
        self.base.pop_coord();
    }

    pub fn locations(&mut self) -> &Locations {
        self.base.locations()
    }

    //

    // TODO add more functions
    pub fn rectangle(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        // self.base.shape_rectangle(x1, y1, x2, y2);
        self.base.locations().set_rectangle(x1, y1, x2, y2);
        self.drawer.draw_quad();
    }
}
